-- Revert game lookup tables and main game tables

DROP INDEX IF EXISTS idx_characters_coat_of_arms;
DROP INDEX IF EXISTS idx_characters_created;
DROP INDEX IF EXISTS idx_characters_name;
DROP INDEX IF EXISTS idx_characters_player;
DROP TABLE IF EXISTS game.characters;

DROP INDEX IF EXISTS idx_players_family_name_unique;
DROP INDEX IF EXISTS idx_players_created;
DROP INDEX IF EXISTS idx_players_family_name;
DROP TABLE IF EXISTS game.players;

DROP TABLE IF EXISTS game.coats_of_arms;
DROP TABLE IF EXISTS game.languages;
