use std::future::Future;
use diesel::Connection;
use diesel::connection::{AnsiTransactionManager, TransactionManager};
use diesel::result::Error;
use tokio::task::spawn_blocking;

// pub async fn transaction<C: Connection, T, E, F>(conn: &mut C, f: F) -> Result<T, E>
//     where
//         F: FnOnce() -> (dyn Future<Output = Result<T, E>>) + Unpin,
//         E: From<Error>
// {
//     let transaction_manager =  spawn_blocking(|| conn.transaction_manager());
//     spawn_blocking(|| transaction_manager.begin_transaction(conn)?);
//     match f().await {
//         Ok(value) => {
//             spawn_blocking(|| transaction_manager.commit_transaction(conn)?);
//             Ok(value)
//         }
//         Err(e) => {
//             spawn_blocking(|| transaction_manager.rollback_transaction(conn)?);
//             Err(e)
//         }
//     }
// }

pub fn finish_transaction<C: Connection, T, E>(conn: &mut C, value: Result<T, E>) -> Result<T, E> {
    match value {
        Ok(value) => {
            AnsiTransactionManager::commit_transaction(conn)?;
            Ok(value)
        }
        Err(e) => {
            AnsiTransactionManager::rollback_transaction(conn).map_err(|e| Error::RollbackError(Box::new(e)))?;
            Err(e)
        }
    }
}