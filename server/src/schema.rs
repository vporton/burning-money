table! {
    payments (id) {
        id -> Int4,
        user_account -> Bytea,
        temp_account_priv_key -> Bytea,
        initiated -> Nullable<Timestamptz>,
        sent_out -> Nullable<Timestamptz>,
    }
}

table! {
    txs (id) {
        id -> Int8,
        user_id -> Int8,
        eth_account -> Bytea,
        usd_amount -> Int8,
        crypto_amount -> Int8,
        status -> Txs_status_type,
        tx_id -> Nullable<Text>,
    }
}

table! {
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

joinable!(txs -> users (user_id));

allow_tables_to_appear_in_same_query!(
    payments,
    txs,
    users,
);
