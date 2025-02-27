
CREATE TABLE accounts (
  id SERIAL PRIMARY KEY,
  username TEXT NOT NULL,
  password TEXT NOT NULL,
  created TIMESTAMPTZ NOT NULL default current_timestamp
)