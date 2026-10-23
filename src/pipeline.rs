use std::fs::File;
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};

use tracing::trace;

use crate::account::report;
use crate::client::dispatcher::ClientDispatcher;
use crate::processing_error::ProcessingError;
use crate::transaction::parser;
use crate::worker_pool::WorkerPool;

/// Reads `path`, applies every transaction, and writes the accounts CSV to `out`.
pub async fn process<W: Write>(path: &Path, out: &mut W) -> Result<(), ProcessingError> {
    let pool = WorkerPool::new();
    let path = PathBuf::from(path);

    let reader = tokio::task::spawn_blocking(move || {
        let result = read_and_dispatch(&path, &pool);
        (pool, result)
    });

    let (pool, dispatch_result) = reader.await.map_err(|error| {
        ProcessingError::WorkerUnavailable(format!("reader task failed: {error}"))
    })?;
    let accounts_result = pool.take_accounts().await;

    match (dispatch_result, accounts_result) {
        (Err(error), _) => Err(error),
        (Ok(()), Err(error)) => Err(error),
        (Ok(()), Ok(accounts)) => report::write_accounts(out, &accounts),
    }
}

fn read_and_dispatch(path: &Path, pool: &WorkerPool) -> Result<(), ProcessingError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let transactions = parser::transactions(reader)?;
    let mut dispatcher = ClientDispatcher::new(pool.worker_count());

    for item in transactions {
        match item {
            Ok(transaction) => {
                let capacities = pool.capacities();
                let worker = dispatcher.worker_for(transaction.client, &capacities);
                pool.dispatch_to(worker, transaction)?;
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
