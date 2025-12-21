-- Migration: Rollback organizations main tables
-- Description: Removes core tables for organization data, treasury, and headquarters

-- Drop triggers
DROP TRIGGER IF EXISTS trigger_validate_organization ON organizations.organizations;
DROP TRIGGER IF EXISTS trigger_update_territory_area_delete ON organizations.territory_cells;
DROP TRIGGER IF EXISTS trigger_update_territory_area_insert ON organizations.territory_cells;
DROP TRIGGER IF EXISTS trigger_update_organization_timestamp ON organizations.organizations;

-- Drop functions
DROP FUNCTION IF EXISTS organizations.validate_organization_constraints();
DROP FUNCTION IF EXISTS organizations.update_territory_area();
DROP FUNCTION IF EXISTS organizations.update_organization_timestamp();

-- Drop tables
DROP TABLE IF EXISTS organizations.treasury_items;
DROP TABLE IF EXISTS organizations.territory_cells;
DROP TABLE IF EXISTS organizations.organizations;
