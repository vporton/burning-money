CREATE TYPE txs_status_type AS ENUM ('before_ordered', 'ordered', 'submitted_to_blockchain', 'confirmed');

CREATE TABLE txs (
    id BIGSERIAL NOT NULL PRIMARY KEY,
    last_modified TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    payment_intent_id TEXT NOT NULL,
    user_id BIGINT NOT NULL,
    eth_account BYTEA NOT NULL,
    usd_amount BIGINT NOT NULL,
    crypto_amount BIGINT NOT NULL,
    bid_date BIGINT NOT NULL,
    status txs_status_type NOT NULL DEFAULT 'before_ordered',
    tx_id BYTEA NULL,
    CONSTRAINT txs_user_fk FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT payment_intent_id_uniq UNIQUE(payment_intent_id)
);
--ALTER TABLE txs ADD CONSTRAINT payment_intent_id_uniq UNIQUE(payment_intent_id);
CREATE INDEX txs_last_modified ON txs USING BTREE(last_modified);
CREATE INDEX txs_user ON txs USING HASH(user_id);
CREATE INDEX txs_tx_id ON txs USING HASH(tx_id);
CREATE INDEX txs_status ON txs USING BTREE(tx_id);

CREATE FUNCTION txs_sync_lastmod() RETURNS trigger AS $$
BEGIN
  NEW.last_modified := NOW();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER
  txs_sync_lastmod
BEFORE UPDATE ON
  txs
FOR EACH ROW EXECUTE PROCEDURE
  txs_sync_lastmod();