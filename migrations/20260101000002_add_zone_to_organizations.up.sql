-- Lien organisation → zone Voronoi (optionnel, backward compatible)
ALTER TABLE organizations.organizations
ADD COLUMN voronoi_zone_id BIGINT REFERENCES terrain.voronoi_zones(id) ON DELETE SET NULL;

CREATE INDEX idx_organizations_zone ON organizations.organizations(voronoi_zone_id);
