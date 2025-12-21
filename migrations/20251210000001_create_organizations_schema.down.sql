-- Migration: Rollback organizations schema and lookup tables
-- Description: Removes the organizations schema structure

-- Drop compatibility table
DROP TABLE IF EXISTS organizations.organization_role_compatibility;

-- Drop lookup tables
DROP TABLE IF EXISTS organizations.role_types;
DROP TABLE IF EXISTS organizations.organization_types;

-- Drop schema
DROP SCHEMA IF EXISTS organizations CASCADE;
