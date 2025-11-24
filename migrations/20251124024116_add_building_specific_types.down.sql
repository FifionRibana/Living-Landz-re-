-- Drop specific type tables
DROP TABLE IF EXISTS buildings.commerce_types;
DROP TABLE IF EXISTS buildings.cult_types;
DROP TABLE IF EXISTS buildings.entertainment_types;
DROP TABLE IF EXISTS buildings.animal_breeding_types;
DROP TABLE IF EXISTS buildings.agriculture_types;
DROP TABLE IF EXISTS buildings.manufacturing_workshop_types;

-- Revert building types to use ManufacturingWorkshop (id=2)
UPDATE buildings.building_types SET specific_type_id = 2 WHERE id IN (10, 20, 21, 22, 23, 30, 40, 50, 51, 52, 53, 54, 55);

-- Remove new building specific types
DELETE FROM buildings.building_specific_types WHERE id IN (3, 4, 5, 6, 7);
