
CREATE TABLE join_character_items (
  character_id INTEGER NOT NULL REFERENCES characters(id),
  item_id INTEGER NOT NULL REFERENCES items(id),
  PRIMARY KEY (character_id, item_id)
)