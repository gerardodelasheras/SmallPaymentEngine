use std::fs::File;
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};

use tokio::sync::mpsc::Sender;
use tracing::trace;

use crate::account::report;
use crate::client::dispatcher::ClientDispatcher;
use crate::processing_error::ProcessingError;
use crate::transaction::parser;
use crate::worker_pool::{Command, WorkerPool};

const MAX_WORKERS: usize = 16;

/// Reads `path`, applies every transaction, and writes the accounts CSV to `out`.
pub async fn process<W: Write>(path: &Path, out: &mut W) -> Result<(), ProcessingError> {
    let workers = std::thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(1)
        .clamp(1, MAX_WORKERS);
    let pool = WorkerPool::spawn(workers);
    let senders = pool.senders();
    let path = PathBuf::from(path);

    let reader = tokio::task::spawn_blocking(move || read_and_dispatch(&path, &senders));
    let dispatch_result = reader
        .await
        .map_err(|error| ProcessingError::WorkerUnavailable(format!("reader task failed: {error}")))
        .and_then(|result| result);

    let accounts_result = pool.take_accounts().await;

    match (dispatch_result, accounts_result) {
        (Err(error), _) => Err(error),
        (Ok(()), Err(error)) => Err(error),
        (Ok(()), Ok(accounts)) => report::write_accounts(out, &accounts),
    }
}

fn read_and_dispatch(path: &Path, senders: &[Sender<Command>]) -> Result<(), ProcessingError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let transactions = parser::transactions(reader)?;
    let mut dispatcher = ClientDispatcher::new(senders.len());

    for item in transactions {
        match item {
            Ok(transaction) => {
                let capacities: Vec<usize> = senders.iter().map(Sender::capacity).collect();
                let worker = dispatcher.worker_for(transaction.client, &capacities);
                senders[worker]
                    .blocking_send(Command::Process(transaction))
                    .map_err(|_| {
                        ProcessingError::WorkerUnavailable(
                            "worker channel closed during dispatch".to_string(),
                        )
                    })?;
            }
            Err(error) => {
                if error.is_recoverable() {
                    trace!(error = %error, "skipped transaction row");
                    continue;
                }
                return Err(error);
            }
        }
    }

    Ok(())
}
