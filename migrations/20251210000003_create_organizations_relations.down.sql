-- Migration: Rollback organizations relations tables
-- Description: Removes members, officers, buildings, and relationships between organizations

-- Drop triggers
DROP TRIGGER IF EXISTS trigger_cleanup_leader_on_removal ON organizations.officers;
DROP TRIGGER IF EXISTS trigger_remove_officer_on_suspension ON organizations.members;
DROP TRIGGER IF EXISTS trigger_validate_officer_role ON organizations.officers;
DROP TRIGGER IF EXISTS trigger_update_population_delete ON organizations.members;
DROP TRIGGER IF EXISTS trigger_update_population_update ON organizations.members;
DROP TRIGGER IF EXISTS trigger_update_population_insert ON organizations.members;

-- Drop functions
DROP FUNCTION IF EXISTS organizations.cleanup_leader_on_officer_removal();
DROP FUNCTION IF EXISTS organizations.remove_officer_on_member_removal();
DROP FUNCTION IF EXISTS organizations.validate_officer_role();
DROP FUNCTION IF EXISTS organizations.update_organization_population();

-- Drop tables
DROP TABLE IF EXISTS organizations.diplomatic_relations;
DROP TABLE IF EXISTS organizations.buildings;
DROP TABLE IF EXISTS organizations.members;
DROP TABLE IF EXISTS organizations.officers;
