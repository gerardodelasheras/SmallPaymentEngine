use std::collections::HashMap;

use crate::account::account::Account;
use crate::client::id::ClientId;
use crate::processing_error::ProcessingError;
use crate::transaction::transaction::Transaction;
use crate::transaction::transaction_entry::{EntryState, TransactionEntry};
use crate::transaction::transaction_id::TransactionId;
use crate::transaction::transaction_type::TransactionType;

/// Per-worker account and transaction state machine.
#[derive(Debug, Default)]
pub struct TransactionProcessor {
    accounts: HashMap<ClientId, Account>,
    entries: HashMap<TransactionId, TransactionEntry>,
}

impl TransactionProcessor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn process(&mut self, transaction: &Transaction) -> Result<(), ProcessingError> {
        // A parsed row initializes its client before domain validation.
        self.accounts.entry(transaction.client).or_default();

        if self
            .accounts
            .get(&transaction.client)
            .map(|account| account.locked)
            .unwrap_or(false)
        {
            return Err(ProcessingError::AccountLocked);
        }

        match transaction.transaction_type {
            TransactionType::Deposit | TransactionType::Withdrawal => {
                self.apply_monetary(transaction)
            }
            TransactionType::Dispute => self.apply_dispute(transaction),
            TransactionType::Resolve => self.apply_resolve(transaction),
            TransactionType::Chargeback => self.apply_chargeback(transaction),
        }
    }

    fn apply_monetary(&mut self, transaction: &Transaction) -> Result<(), ProcessingError> {
        let amount = transaction.amount.ok_or(ProcessingError::MissingAmount)?;
        if self.entries.contains_key(&transaction.transaction_id) {
            return Err(ProcessingError::DuplicateTransaction(
                transaction.transaction_id,
            ));
        }

        let account = self
            .accounts
            .get_mut(&transaction.client)
            .expect("account initialized");
        match transaction.transaction_type {
            TransactionType::Deposit => account.credit(amount)?,
            TransactionType::Withdrawal => account.debit(amount)?,
            _ => unreachable!("apply_monetary only handles monetary transactions"),
        }

        self.entries.insert(
            transaction.transaction_id,
            TransactionEntry {
                client: transaction.client,
                amount,
                kind: transaction.transaction_type,
                state: EntryState::Settled,
            },
        );
        Ok(())
    }

    fn lookup(&self, transaction: &Transaction) -> Result<TransactionEntry, ProcessingError> {
        let entry = self.entries.get(&transaction.transaction_id).ok_or(
            ProcessingError::UnknownTransaction(transaction.transaction_id),
        )?;
        if entry.client != transaction.client {
            return Err(ProcessingError::NotOwner(transaction.transaction_id));
        }
        Ok(*entry)
    }

    fn apply_dispute(&mut self, transaction: &Transaction) -> Result<(), ProcessingError> {
        let entry = self.lookup(transaction)?;
        if entry.state == EntryState::Disputed {
            return Err(ProcessingError::AlreadyDisputed(transaction.transaction_id));
        }

        let account = self
            .accounts
            .get_mut(&transaction.client)
            .expect("account initialized");
        account.hold(entry.amount)?;
        self.entries
            .get_mut(&transaction.transaction_id)
            .expect("entry exists")
            .state = EntryState::Disputed;
        Ok(())
    }

    fn apply_resolve(&mut self, transaction: &Transaction) -> Result<(), ProcessingError> {
        let entry = self.lookup(transaction)?;
        if entry.state != EntryState::Disputed {
            return Err(ProcessingError::NotDisputed(transaction.transaction_id));
        }

        let account = self
            .accounts
            .get_mut(&transaction.client)
            .expect("account initialized");
        account.release(entry.amount)?;
        self.entries
            .get_mut(&transaction.transaction_id)
            .expect("entry exists")
            .state = EntryState::Settled;
        Ok(())
    }

    fn apply_chargeback(&mut self, transaction: &Transaction) -> Result<(), ProcessingError> {
        let entry = self.lookup(transaction)?;
        if entry.state != EntryState::Disputed {
            return Err(ProcessingError::NotDisputed(transaction.transaction_id));
        }

        let account = self
            .accounts
            .get_mut(&transaction.client)
            .expect("account initialized");
        account.chargeback(entry.amount)?;
        self.entries.remove(&transaction.transaction_id);
        Ok(())
    }

    pub fn accounts(&self) -> &HashMap<ClientId, Account> {
        &self.accounts
    }

    pub fn snapshot(&self) -> HashMap<ClientId, Account> {
        self.accounts.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::amount::Amount;

    fn amount(value: &str) -> Option<Amount> {
        Some(Amount::parse(value).expect("valid amount"))
    }

    fn transaction(
        transaction_type: TransactionType,
        client: ClientId,
        id: TransactionId,
        value: Option<&str>,
    ) -> Transaction {
        Transaction {
            transaction_type,
            client,
            transaction_id: id,
            amount: value.map(|text| Amount::parse(text).expect("valid amount")),
        }
    }

    fn deposit(
        processor: &mut TransactionProcessor,
        client: ClientId,
        id: TransactionId,
        value: &str,
    ) {
        processor
            .process(&transaction(
                TransactionType::Deposit,
                client,
                id,
                Some(value),
            ))
            .expect("deposit succeeds");
    }

    fn account(processor: &TransactionProcessor, client: ClientId) -> Account {
        processor
            .accounts()
            .get(&client)
            .cloned()
            .expect("account exists")
    }

    #[test]
    fn deposit_credits_available() {
        let mut processor = TransactionProcessor::new();
        deposit(&mut processor, 1, 1, "10.0");
        assert_eq!(account(&processor, 1).available, amount("10.0").unwrap());
        assert_eq!(account(&processor, 1).total(), amount("10.0").unwrap());
    }

    #[test]
    fn withdrawal_debits_available() {
        let mut processor = TransactionProcessor::new();
        deposit(&mut processor, 1, 1, "10.0");
        processor
            .process(&transaction(TransactionType::Withdrawal, 1, 2, Some("4.0")))
            .expect("withdrawal succeeds");
        assert_eq!(account(&processor, 1).available, amount("6.0").unwrap());
    }

    #[test]
    fn insufficient_withdrawal_leaves_balances_unchanged() {
        let mut processor = TransactionProcessor::new();
        deposit(&mut processor, 1, 1, "3.0");
        assert!(matches!(
            processor.process(&transaction(TransactionType::Withdrawal, 1, 2, Some("5.0"))),
            Err(ProcessingError::InsufficientFunds)
        ));
        assert_eq!(account(&processor, 1).available, amount("3.0").unwrap());
    }

    #[test]
    fn dispute_holds_funds() {
        let mut processor = TransactionProcessor::new();
        deposit(&mut processor, 1, 1, "10.0");
        processor
            .process(&transaction(TransactionType::Dispute, 1, 1, None))
            .expect("dispute succeeds");
        let account = account(&processor, 1);
        assert_eq!(account.available, Amount::ZERO);
        assert_eq!(account.held, amount("10.0").unwrap());
        assert_eq!(account.total(), amount("10.0").unwrap());
    }

    #[test]
    fn dispute_may_overdraw_available() {
        let mut processor = TransactionProcessor::new();
        deposit(&mut processor, 1, 1, "100.0");
        processor
            .process(&transaction(
                TransactionType::Withdrawal,
                1,
                2,
                Some("80.0"),
            ))
            .expect("withdrawal succeeds");
        processor
            .process(&transaction(TransactionType::Dispute, 1, 1, None))
            .expect("dispute succeeds");
        let account = account(&processor, 1);
        assert_eq!(account.available.to_string(), "-80.0");
        assert_eq!(account.held, amount("100.0").unwrap());
        assert_eq!(account.total(), amount("20.0").unwrap());
    }

    #[test]
    fn dispute_of_unknown_transaction_is_rejected() {
        let mut processor = TransactionProcessor::new();
        assert!(matches!(
            processor.process(&transaction(TransactionType::Dispute, 1, 99, None)),
            Err(ProcessingError::UnknownTransaction(99))
        ));
        assert_eq!(account(&processor, 1).available, Amount::ZERO);
    }

    #[test]
    fn duplicate_dispute_is_rejected() {
        let mut processor = TransactionProcessor::new();
        deposit(&mut processor, 1, 1, "10.0");
        processor
            .process(&transaction(TransactionType::Dispute, 1, 1, None))
            .expect("first dispute succeeds");
        assert!(matches!(
            processor.process(&transaction(TransactionType::Dispute, 1, 1, None)),
            Err(ProcessingError::AlreadyDisputed(1))
        ));
        assert_eq!(account(&processor, 1).held, amount("10.0").unwrap());
    }

    #[test]
    fn resolve_releases_held_funds() {
        let mut processor = TransactionProcessor::new();
        deposit(&mut processor, 1, 1, "10.0");
        processor
            .process(&transaction(TransactionType::Dispute, 1, 1, None))
            .unwrap();
        processor
            .process(&transaction(TransactionType::Resolve, 1, 1, None))
            .unwrap();
        let account = account(&processor, 1);
        assert_eq!(account.available, amount("10.0").unwrap());
        assert_eq!(account.held, Amount::ZERO);
    }

    #[test]
    fn resolving_a_settled_transaction_is_rejected() {
        let mut processor = TransactionProcessor::new();
        deposit(&mut processor, 1, 1, "10.0");
        assert!(matches!(
            processor.process(&transaction(TransactionType::Resolve, 1, 1, None)),
            Err(ProcessingError::NotDisputed(1))
        ));
    }

    #[test]
    fn resolved_transaction_may_be_disputed_again() {
        let mut processor = TransactionProcessor::new();
        deposit(&mut processor, 1, 1, "10.0");
        processor
            .process(&transaction(TransactionType::Dispute, 1, 1, None))
            .unwrap();
        processor
            .process(&transaction(TransactionType::Resolve, 1, 1, None))
            .unwrap();
        processor
            .process(&transaction(TransactionType::Dispute, 1, 1, None))
            .unwrap();
        assert_eq!(account(&processor, 1).held, amount("10.0").unwrap());
    }

    #[test]
    fn chargeback_removes_held_funds_and_locks() {
        let mut processor = TransactionProcessor::new();
        deposit(&mut processor, 1, 1, "10.0");
        processor
            .process(&transaction(TransactionType::Dispute, 1, 1, None))
            .unwrap();
        processor
            .process(&transaction(TransactionType::Chargeback, 1, 1, None))
            .unwrap();
        let account = account(&processor, 1);
        assert_eq!(account.available, Amount::ZERO);
        assert_eq!(account.held, Amount::ZERO);
        assert_eq!(account.total(), Amount::ZERO);
        assert!(account.locked);
    }

    #[test]
    fn chargeback_of_settled_transaction_is_rejected() {
        let mut processor = TransactionProcessor::new();
        deposit(&mut processor, 1, 1, "10.0");
        assert!(matches!(
            processor.process(&transaction(TransactionType::Chargeback, 1, 1, None)),
            Err(ProcessingError::NotDisputed(1))
        ));
        assert!(!account(&processor, 1).locked);
    }

    #[test]
    fn locked_account_rejects_later_operations() {
        let mut processor = TransactionProcessor::new();
        deposit(&mut processor, 1, 1, "10.0");
        processor
            .process(&transaction(TransactionType::Dispute, 1, 1, None))
            .unwrap();
        processor
            .process(&transaction(TransactionType::Chargeback, 1, 1, None))
            .unwrap();
        assert!(matches!(
            processor.process(&transaction(TransactionType::Deposit, 1, 2, Some("5.0"))),
            Err(ProcessingError::AccountLocked)
        ));
        assert_eq!(account(&processor, 1).available, Amount::ZERO);
    }

    #[test]
    fn duplicate_transaction_identifier_is_rejected() {
        let mut processor = TransactionProcessor::new();
        deposit(&mut processor, 1, 1, "10.0");
        assert!(matches!(
            processor.process(&transaction(TransactionType::Deposit, 1, 1, Some("5.0"))),
            Err(ProcessingError::DuplicateTransaction(1))
        ));
        assert_eq!(account(&processor, 1).available, amount("10.0").unwrap());
    }

    #[test]
    fn foreign_transaction_reference_is_rejected() {
        let mut processor = TransactionProcessor::new();
        deposit(&mut processor, 1, 1, "10.0");
        assert!(matches!(
            processor.process(&transaction(TransactionType::Dispute, 2, 1, None)),
            Err(ProcessingError::NotOwner(1))
        ));
        assert_eq!(account(&processor, 1).available, amount("10.0").unwrap());
        assert_eq!(account(&processor, 2).held, Amount::ZERO);
    }

    #[test]
    fn rejected_row_still_initializes_the_client() {
        let mut processor = TransactionProcessor::new();
        assert!(matches!(
            processor.process(&transaction(TransactionType::Withdrawal, 1, 1, Some("5.0"))),
            Err(ProcessingError::InsufficientFunds)
        ));
        assert_eq!(account(&processor, 1), Account::default());
    }
}
