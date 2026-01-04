-- Remove zone reference from organizations
DROP INDEX IF EXISTS organizations.idx_organizations_zone;
ALTER TABLE organizations.organizations DROP COLUMN IF EXISTS voronoi_zone_id;
