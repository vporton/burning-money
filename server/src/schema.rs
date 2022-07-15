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
    users (id) {
        id -> Int4,
        first_name -> Text,
        last_name -> Text,
        email -> Text,
        created_at -> Timestamp,
    }
}

allow_tables_to_appear_in_same_query!(
    payments,
    users,
);
