-- Remove Unknown entry from building_types
DELETE FROM buildings.building_types WHERE id = 0;
