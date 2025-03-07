
CREATE OR REPLACE FUNCTION update_modified_column()
RETURNS TRIGGER AS $$
BEGIN
   IF row(NEW.*) IS DISTINCT FROM row(OLD.*) THEN
      NEW.modified = now(); 
      RETURN NEW;
   ELSE
      RETURN OLD;
   END IF;
END;
$$ language 'plpgsql';

ALTER TABLE characters ADD COLUMN modified TIMESTAMPTZ NOT NULL DEFAULT current_timestamp;
CREATE TRIGGER update_character_modified BEFORE UPDATE ON characters FOR EACH ROW EXECUTE PROCEDURE update_modified_column();