-- Add down migration script here
DROP TABLE IF EXISTS game.characters CASCADE;
DROP TABLE IF EXISTS game.players CASCADE;
DROP TABLE IF EXISTS game.coats_of_arms CASCADE;
DROP TABLE IF EXISTS game.languages CASCADE;