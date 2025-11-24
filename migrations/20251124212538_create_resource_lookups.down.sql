-- Revert resource lookup tables

DROP INDEX IF EXISTS idx_resource_types_specific;
DROP INDEX IF EXISTS idx_resource_types_category;
DROP TABLE IF EXISTS resources.resource_types;
DROP TABLE IF EXISTS resources.resource_specific_types;
DROP TABLE IF EXISTS resources.resource_categories;
