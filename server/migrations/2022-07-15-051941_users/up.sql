CREATE TABLE users (
  id BIGSERIAL NOT NULL PRIMARY KEY,
  first_name TEXT NOT NULL,
  last_name TEXT NOT NULL,
  email TEXT NOT NULL,
  password TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL,
  CONSTRAINT unique_email UNIQUE (email)
);
