use crate::sql_types::TxsStatusType;

pub struct Tx {
    pub id: i64,
    pub user_id: i64,
    pub eth_account: Vec<u8>,
    pub usd_amount: i64,
    pub crypto_amount: i64,
    pub bid_day: i64,
    pub status: TxsStatusType,
    pub tx_id: Option<Vec<u8>>,
}
