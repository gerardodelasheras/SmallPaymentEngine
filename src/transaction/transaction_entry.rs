use crate::amount::Amount;
use crate::client::id::ClientId;
use crate::transaction::transaction_type::TransactionType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryState {
    Settled,
    Disputed,
}

/// An accepted monetary transaction kept for later dispute operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransactionEntry {
    pub client: ClientId,
    pub amount: Amount,
    pub kind: TransactionType,
    pub state: EntryState,
}
