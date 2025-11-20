-- Add down migration script here
DROP TABLE IF EXISTS terrain.cells CASCADE;
DROP TABLE IF EXISTS terrain.terrain_biomes CASCADE;
DROP TABLE IF EXISTS terrain.terrains CASCADE;