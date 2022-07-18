CREATE TYPE txs_status_type AS ENUM ('created', 'submitted_to_blockchain', 'confirmed');

CREATE TABLE txs (
    id BIGSERIAL NOT NULL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    eth_account BYTEA NOT NULL,
    usd_amount BIGINT NOT NULL,
    crypto_amount BIGINT NOT NULL,
    status txs_status_type NOT NULL DEFAULT 'created',
    tx_id TEXT NULL, -- Ethereum tx ID.
    CONSTRAINT txs_user_fk FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
);
CREATE INDEX txs_user ON txs USING HASH(user_id);
CREATE INDEX txs_tx_id ON txs USING HASH(tx_id);
CREATE INDEX txs_status ON txs USING BTREE(tx_id);
