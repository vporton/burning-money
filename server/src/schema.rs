// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "txs_status_type"))]
    pub struct TxsStatusType;
}

diesel::table! {
    payments (id) {
        id -> Int4,
        user_account -> Bytea,
        temp_account_priv_key -> Bytea,
        initiated -> Nullable<Timestamptz>,
        sent_out -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::TxsStatusType;

    txs (id) {
        id -> Int8,
        user_id -> Int8,
        eth_account -> Bytea,
        usd_amount -> Int8,
        crypto_amount -> Int8,
        status -> TxsStatusType,
        tx_id -> Nullable<Text>,
    }
}

diesel::table! {
    users (id) {
        id -> Int8,
        first_name -> Text,
        last_name -> Text,
        email -> Text,
        password -> Text,
        created_at -> Timestamp,
        passed_kyc -> Bool,
    }
}

diesel::joinable!(txs -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    payments,
    txs,
    users,
);
