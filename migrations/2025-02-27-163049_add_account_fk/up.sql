
-- add account id to characters
ALTER TABLE characters ADD account_id INTEGER NOT NULL DEFAULT 0;

-- add foreign key constraint to account_id
ALTER TABLE characters ADD CONSTRAINT characters_account_id_fkey FOREIGN KEY (account_id) REFERENCES accounts(id);