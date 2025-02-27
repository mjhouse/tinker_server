
CREATE TABLE players (
  id SERIAL PRIMARY KEY,
  "name" TEXT NOT NULL,
  created TIMESTAMPTZ NOT NULL default current_timestamp
)