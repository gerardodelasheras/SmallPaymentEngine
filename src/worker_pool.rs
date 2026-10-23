use std::collections::HashMap;

use tokio::sync::{mpsc, oneshot};
use tracing::{error, trace};

use crate::account::account::Account;
use crate::client::id::ClientId;
use crate::processing_error::ProcessingError;
use crate::transaction::transaction::Transaction;
use crate::transaction::transaction_processor::TransactionProcessor;

/// Bounded per-worker queue capacity.
pub const QUEUE_CAPACITY: usize = 256;

pub enum Command {
    Process(Transaction),
    Snapshot(oneshot::Sender<HashMap<ClientId, Account>>),
    Shutdown,
}

/// A fixed pool of workers, each owning a disjoint set of clients.
pub struct WorkerPool {
    senders: Vec<mpsc::Sender<Command>>,
    handles: Vec<tokio::task::JoinHandle<()>>,
}

impl WorkerPool {
    pub fn spawn(workers: usize) -> Self {
        assert!(workers > 0, "at least one worker is required");
        let mut senders = Vec::with_capacity(workers);
        let mut handles = Vec::with_capacity(workers);
        for _ in 0..workers {
            let (sender, receiver) = mpsc::channel(QUEUE_CAPACITY);
            senders.push(sender);
            handles.push(tokio::spawn(run_worker(receiver)));
        }
        Self { senders, handles }
    }

    pub fn senders(&self) -> Vec<mpsc::Sender<Command>> {
        self.senders.clone()
    }

    pub fn sender(&self, worker: usize) -> mpsc::Sender<Command> {
        self.senders[worker].clone()
    }

    pub fn capacities(&self) -> Vec<usize> {
        self.senders.iter().map(mpsc::Sender::capacity).collect()
    }

    /// Blocks the calling (non-async) thread when a worker queue is full.
    pub fn send_blocking(&self, worker: usize, command: Command) -> Result<(), ProcessingError> {
        self.senders[worker]
            .blocking_send(command)
            .map_err(|_| ProcessingError::WorkerUnavailable("worker channel closed".to_string()))
    }

    /// Collects every worker's accounts, then shuts the workers down.
    pub async fn take_accounts(mut self) -> Result<HashMap<ClientId, Account>, ProcessingError> {
        let mut merged = HashMap::new();
        for sender in &self.senders {
            let (reply, receiver) = oneshot::channel();
            sender.send(Command::Snapshot(reply)).await.map_err(|_| {
                ProcessingError::WorkerUnavailable("snapshot request failed".to_string())
            })?;
            match receiver.await {
                Ok(accounts) => merged.extend(accounts),
                Err(_) => {
                    return Err(ProcessingError::WorkerUnavailable(
                        "snapshot reply dropped".to_string(),
                    ))
                }
            }
        }

        for sender in &self.senders {
            let _ = sender.send(Command::Shutdown).await;
        }
        for handle in self.handles.drain(..) {
            let _ = handle.await;
        }
        Ok(merged)
    }
}

async fn run_worker(mut receiver: mpsc::Receiver<Command>) {
    let mut processor = TransactionProcessor::new();
    while let Some(command) = receiver.recv().await {
        match command {
            Command::Process(transaction) => {
                if let Err(problem) = processor.process(&transaction) {
                    if problem.is_recoverable() {
                        trace!(error = %problem, "skipped transaction row");
                    } else {
                        error!(error = %problem, "worker infrastructure error");
                    }
                }
            }
            Command::Snapshot(reply) => {
                let _ = reply.send(processor.snapshot());
            }
            Command::Shutdown => break,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::amount::Amount;
    use crate::transaction::transaction_type::TransactionType;
    use tokio::sync::mpsc::error::TrySendError;

    fn deposit(client: ClientId, id: u32, value: &str) -> Transaction {
        Transaction {
            transaction_type: TransactionType::Deposit,
            client,
            transaction_id: id,
            amount: Some(Amount::parse(value).expect("valid amount")),
        }
    }

    #[tokio::test]
    async fn queues_start_at_the_bounded_capacity() {
        let pool = WorkerPool::spawn(2);
        assert_eq!(pool.capacities(), vec![QUEUE_CAPACITY, QUEUE_CAPACITY]);
        let accounts = pool.take_accounts().await.expect("pool drains");
        assert!(accounts.is_empty());
    }

    #[tokio::test]
    async fn snapshot_includes_processed_transactions() {
        let pool = WorkerPool::spawn(1);
        pool.sender(0)
            .send(Command::Process(deposit(1, 1, "10.0")))
            .await
            .expect("dispatch succeeds");

        let accounts = pool.take_accounts().await.expect("pool drains");
        assert_eq!(accounts[&1].available.to_string(), "10.0");
    }

    #[tokio::test]
    async fn a_full_queue_exposes_backpressure() {
        let pool = WorkerPool::spawn(1);
        let sender = pool.sender(0);
        for id in 1..=QUEUE_CAPACITY as u32 {
            sender
                .try_send(Command::Process(deposit(1, id, "1.0")))
                .expect("queue has capacity");
        }
        let overflow = sender.try_send(Command::Process(deposit(1, 9_999, "1.0")));
        assert!(matches!(overflow, Err(TrySendError::Full(_))));

        let accounts = pool.take_accounts().await.expect("pool drains");
        assert_eq!(accounts[&1].available.to_string(), "256.0");
    }

    #[tokio::test]
    async fn snapshot_reports_a_stopped_worker() {
        let pool = WorkerPool::spawn(1);
        pool.sender(0)
            .send(Command::Shutdown)
            .await
            .expect("shutdown is sent");
        assert!(pool.take_accounts().await.is_err());
    }

    #[tokio::test]
    async fn a_rejected_row_does_not_stop_the_worker() {
        let pool = WorkerPool::spawn(1);
        let sender = pool.sender(0);
        sender
            .send(Command::Process(deposit(1, 1, "5.0")))
            .await
            .expect("dispatch succeeds");
        sender
            .send(Command::Process(Transaction {
                transaction_type: TransactionType::Withdrawal,
                client: 1,
                transaction_id: 2,
                amount: Some(Amount::parse("100.0").expect("valid amount")),
            }))
            .await
            .expect("dispatch succeeds");
        sender
            .send(Command::Process(deposit(1, 3, "7.0")))
            .await
            .expect("dispatch succeeds");

        let accounts = pool.take_accounts().await.expect("pool drains");
        assert_eq!(accounts[&1].available.to_string(), "12.0");
    }
}
