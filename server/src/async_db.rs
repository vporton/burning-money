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

pub async fn finish_transaction<T, E: From<tokio_postgres::Error>>(
    trans: tokio_postgres::Transaction<'_>,
    value: Result<T, E>
) -> Result<T, E> {
    match value {
        Ok(value) => {
            trans.commit().await?;
            Ok(value)
        }
        Err(e) => {
            trans.rollback().await?;
            Err(e)
        }
    }
}