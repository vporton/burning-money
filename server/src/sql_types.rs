use postgres_types::{ToSql, FromSql};

#[derive(Debug, ToSql, FromSql)]
#[postgres(name = "txs_status_type")]
pub enum TxsStatusType {
    #[postgres(name = "before_ordered")]
    BeforeOrdered,
    #[postgres(name = "ordered")]
    Ordered,
    #[postgres(name = "submitted_to_blockchain")]
    SubmittedToBlockchain,
    #[postgres(name = "confirmed")]
    Confirmed,
}