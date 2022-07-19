#[derive(diesel_derive_enum::DbEnum, Debug)]
#[DieselTypePath = "crate::schema::sql_types::TxsStatusType"]
pub enum TxsStatusType {
    Created,
    SubmittedToBlockchain,
    Confirmed,
}