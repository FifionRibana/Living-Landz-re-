-- Revert terrain tables

DROP INDEX IF EXISTS idx_cells_building;
DROP INDEX IF EXISTS idx_cells_chunk;
DROP TABLE IF EXISTS terrain.cells;

DROP INDEX IF EXISTS idx_terrain_biomes_generated;
DROP TABLE IF EXISTS terrain.terrain_biomes;

DROP INDEX IF EXISTS idx_terrains_generated;
DROP TABLE IF EXISTS terrain.terrains;
