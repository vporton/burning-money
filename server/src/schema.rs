table! {
    payments (id) {
        id -> Int4,
        user_account -> Bytea,
        temp_account_priv_key -> Bytea,
        initiated -> Nullable<Timestamptz>,
        sent_out -> Nullable<Timestamptz>,
    }
}
