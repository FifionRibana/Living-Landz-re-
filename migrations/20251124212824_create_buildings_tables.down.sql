-- Revert buildings tables

-- Remove foreign key constraint from terrain.cells first
ALTER TABLE terrain.cells
DROP CONSTRAINT IF EXISTS cells_building_id_fkey;

-- Drop specific building tables
DROP INDEX IF EXISTS idx_building_commerce;
DROP TABLE IF EXISTS buildings.commerce;

DROP INDEX IF EXISTS idx_building_cult;
DROP TABLE IF EXISTS buildings.cult;

DROP INDEX IF EXISTS idx_building_entertainment;
DROP TABLE IF EXISTS buildings.entertainment;

DROP INDEX IF EXISTS idx_building_animal_breeding;
DROP TABLE IF EXISTS buildings.animal_breeding;

DROP INDEX IF EXISTS idx_building_agriculture;
DROP TABLE IF EXISTS buildings.agriculture;

DROP INDEX IF EXISTS idx_building_manufacturing_workshops;
DROP TABLE IF EXISTS buildings.manufacturing_workshops;

DROP INDEX IF EXISTS idx_building_trees;
DROP TABLE IF EXISTS buildings.trees;

-- Drop base buildings table
DROP INDEX IF EXISTS idx_buildings_is_built;
DROP INDEX IF EXISTS idx_buildings_created;
DROP INDEX IF EXISTS idx_buildings_chunk;
DROP INDEX IF EXISTS idx_building_type_id;
DROP TABLE IF EXISTS buildings.buildings_base;
