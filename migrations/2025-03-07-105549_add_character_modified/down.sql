
DROP TRIGGER update_character_modified ON characters;
DROP FUNCTION update_modified_column;
ALTER TABLE characters DROP COLUMN modified;