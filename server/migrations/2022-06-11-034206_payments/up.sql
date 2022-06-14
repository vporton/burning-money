CREATE TABLE payments (
    id SERIAL PRIMARY KEY,
    user_account BYTEA NOT NULL,
    temp_account_priv_key BYTEA NOT NULL, -- encrypted
    initiated TIMESTAMPTZ NULL DEFAULT CURRENT_TIMESTAMP,
    sent_out TIMESTAMPTZ NULL
);
ALTER TABLE payments ADD CONSTRAINT user_account_uniq UNIQUE (user_account);
CREATE INDEX temp_account_priv_key_index ON payments USING BTREE(temp_account_priv_key);
