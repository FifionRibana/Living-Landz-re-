-- Add down migration script here
DROP TABLE IF EXISTS resources.resource_types CASCADE;
DROP TABLE IF EXISTS resources.resource_specific_types CASCADE;
DROP TABLE IF EXISTS resources.resource_categories CASCADE;