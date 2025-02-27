
CREATE TABLE join_character_abilities (
  character_id INTEGER NOT NULL REFERENCES characters(id),
  ability_id INTEGER NOT NULL REFERENCES abilities(id),
  PRIMARY KEY (character_id, ability_id)
)