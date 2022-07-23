use postgres_types::{ToSql, FromSql};

#[derive(Debug, ToSql, FromSql)]
#[postgres(name = "txs_status_type")]
pub enum TxsStatusType {
    #[postgres(name = "created")]
    Created,
    #[postgres(name = "submitted_to_blockchain")]
    SubmittedToBlockchain,
    #[postgres(name = "confirmed")]
    Confirmed,
}