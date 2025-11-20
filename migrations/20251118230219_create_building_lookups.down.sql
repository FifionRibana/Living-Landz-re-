-- Add down migration script here
DROP TABLE IF EXISTS buildings.building_types CASCADE;
DROP TABLE IF EXISTS buildings.tree_types CASCADE;
DROP TABLE IF EXISTS buildings.building_specific_types CASCADE;
DROP TABLE IF EXISTS buildings.building_categories CASCADE;