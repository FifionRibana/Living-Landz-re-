-- Add down migration script here
DROP INDEX IF EXISTS buildings.idx_buildings_is_built;

ALTER TABLE buildings.buildings_base
DROP COLUMN is_built;
