-- Revert building lookup tables

-- Drop building_types first (has FK to other tables)
DROP INDEX IF EXISTS idx_building_types_specific;
DROP INDEX IF EXISTS idx_building_types_category;
DROP TABLE IF EXISTS buildings.building_types;

-- Drop specific type lookup tables
DROP TABLE IF EXISTS buildings.commerce_types;
DROP TABLE IF EXISTS buildings.cult_types;
DROP TABLE IF EXISTS buildings.entertainment_types;
DROP TABLE IF EXISTS buildings.animal_breeding_types;
DROP TABLE IF EXISTS buildings.agriculture_types;
DROP TABLE IF EXISTS buildings.manufacturing_workshop_types;
DROP TABLE IF EXISTS buildings.tree_types;

-- Drop main lookup tables
DROP TABLE IF EXISTS buildings.building_specific_types;
DROP TABLE IF EXISTS buildings.building_categories;
