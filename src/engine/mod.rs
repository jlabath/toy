use crate::account::Account;
use crate::transaction::Transaction;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

/// Starts the transaction engine, returning tuple of items that can be used to communicate with it.
///
/// Handle that can be awaited on to yield HashMap of accounts when the transaction engine finishes.
///
/// Channel that can be used to send Transactions to the engine.
/// Clone the channel if multiple producers are desired.
///
/// The engine loop will end when all producer channels are dropped.
pub async fn start() -> (JoinHandle<HashMap<u16, Account>>, mpsc::Sender<Transaction>) {
    let (tx, rx) = mpsc::channel(100);

    let handle = tokio::spawn(run_engine(rx));

    (handle, tx)
}

/// The engine loop itself.
///
/// HashMap (memory) is used to store the accounts.
async fn run_engine(mut rx: mpsc::Receiver<Transaction>) -> HashMap<u16, Account> {
    let mut storage: HashMap<u16, Account> = HashMap::new();
    while let Some(tx) = rx.recv().await {
        match storage.get_mut(&tx.client_id) {
            Some(acc) => {
                match acc.add_transaction(tx) {
                    Ok(()) => (),
                    Err(_e) => (), //normally we'd log this somewhere
                }
            }
            None => {
                let mut acc = Account::new(tx.client_id);
                match acc.add_transaction(tx) {
                    Ok(()) => (),
                    Err(_e) => (), //normally we'd log this somewhere
                }
                storage.insert(acc.client_id, acc);
            }
        }
    }
    //return the accounts
    storage
}

#[cfg(test)]
mod tests;
