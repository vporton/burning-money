use std::future::Future;
use diesel::Connection;
use diesel::result::Error;
use tokio::task::spawn_blocking;

async fn transaction<C: Connection, T, E, F>(conn: C, f: F) -> Result<T, E>
    where
        F: FnOnce() -> dyn Future<Output = Result<T, E>>,
        E: From<Error>
{
    let transaction_manager =  spawn_blocking(|| conn.transaction_manager());
    spawn_blocking(|| transaction_manager.begin_transaction(conn)?);
    match f().await {
        Ok(value) => {
            spawn_blocking(|| transaction_manager.commit_transaction(conn)?);
            Ok(value)
        }
        Err(e) => {
            spawn_blocking(|| transaction_manager.rollback_transaction(conn)?);
            Err(e)
        }
    }
}
