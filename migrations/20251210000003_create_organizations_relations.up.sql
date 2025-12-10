-- Migration: Create organizations relations tables
-- Description: Members, officers, buildings, and relationships between organizations

-- ============================================================================
-- OFFICERS (Postes importants)
-- ============================================================================

CREATE TABLE organizations.officers (
    id BIGSERIAL PRIMARY KEY,
    organization_id BIGINT NOT NULL REFERENCES organizations.organizations(id) ON DELETE CASCADE,
    unit_id BIGINT NOT NULL REFERENCES units.units(id) ON DELETE CASCADE,
    role_type_id SMALLINT NOT NULL REFERENCES organizations.role_types(id),
    appointed_at TIMESTAMPTZ DEFAULT NOW(),
    appointed_by_unit_id BIGINT REFERENCES units.units(id) ON DELETE SET NULL,

    -- A unit can only hold one role per organization
    UNIQUE (organization_id, unit_id, role_type_id)
);

CREATE INDEX idx_officers_org ON organizations.officers(organization_id);
CREATE INDEX idx_officers_unit ON organizations.officers(unit_id);
CREATE INDEX idx_officers_role ON organizations.officers(role_type_id);

COMMENT ON TABLE organizations.officers IS 'Officiers et postes importants dans les organisations';
COMMENT ON COLUMN organizations.officers.appointed_by_unit_id IS 'ID de l''unité qui a nommé cet officier';

-- ============================================================================
-- MEMBERS (Membres de l'organisation)
-- ============================================================================

CREATE TABLE organizations.members (
    id BIGSERIAL PRIMARY KEY,
    organization_id BIGINT NOT NULL REFERENCES organizations.organizations(id) ON DELETE CASCADE,
    unit_id BIGINT NOT NULL REFERENCES units.units(id) ON DELETE CASCADE,
    joined_at TIMESTAMPTZ DEFAULT NOW(),
    invited_by_unit_id BIGINT REFERENCES units.units(id) ON DELETE SET NULL,
    membership_status VARCHAR(20) DEFAULT 'active' CHECK (membership_status IN ('active', 'suspended', 'honorary')),

    -- A unit can only be a member once per organization
    UNIQUE (organization_id, unit_id)
);

CREATE INDEX idx_members_org ON organizations.members(organization_id);
CREATE INDEX idx_members_unit ON organizations.members(unit_id);
CREATE INDEX idx_members_status ON organizations.members(membership_status);

COMMENT ON TABLE organizations.members IS 'Membres des organisations';
COMMENT ON COLUMN organizations.members.membership_status IS 'Statut: active, suspended, honorary';
COMMENT ON COLUMN organizations.members.invited_by_unit_id IS 'ID de l''unité qui a invité ce membre';

-- ============================================================================
-- BUILDINGS (Bâtiments possédés)
-- ============================================================================

CREATE TABLE organizations.buildings (
    id BIGSERIAL PRIMARY KEY,
    organization_id BIGINT NOT NULL REFERENCES organizations.organizations(id) ON DELETE CASCADE,
    building_id BIGINT NOT NULL, -- Reference to buildings table (to be created later)
    acquired_at TIMESTAMPTZ DEFAULT NOW(),
    acquired_by_unit_id BIGINT REFERENCES units.units(id) ON DELETE SET NULL,
    building_role VARCHAR(50), -- 'headquarters', 'warehouse', 'barracks', 'temple', etc.

    -- A building can only belong to one organization
    UNIQUE (building_id)
);

CREATE INDEX idx_org_buildings_org ON organizations.buildings(organization_id);
CREATE INDEX idx_org_buildings_building ON organizations.buildings(building_id);
CREATE INDEX idx_org_buildings_role ON organizations.buildings(building_role);

COMMENT ON TABLE organizations.buildings IS 'Bâtiments possédés par les organisations';
COMMENT ON COLUMN organizations.buildings.building_role IS 'Rôle du bâtiment: headquarters, warehouse, barracks, etc.';
COMMENT ON COLUMN organizations.buildings.acquired_by_unit_id IS 'ID de l''unité qui a acquis ce bâtiment';

-- ============================================================================
-- DIPLOMATIC RELATIONS (Relations entre organisations)
-- ============================================================================

CREATE TABLE organizations.diplomatic_relations (
    id BIGSERIAL PRIMARY KEY,
    organization_id BIGINT NOT NULL REFERENCES organizations.organizations(id) ON DELETE CASCADE,
    target_organization_id BIGINT NOT NULL REFERENCES organizations.organizations(id) ON DELETE CASCADE,
    relation_type VARCHAR(20) NOT NULL CHECK (relation_type IN ('allied', 'neutral', 'hostile', 'at_war', 'trade_agreement', 'non_aggression')),
    established_at TIMESTAMPTZ DEFAULT NOW(),
    established_by_unit_id BIGINT REFERENCES units.units(id) ON DELETE SET NULL,
    expires_at TIMESTAMPTZ, -- NULL = permanent

    -- Can't have relation with self
    CHECK (organization_id != target_organization_id),

    -- Ensure unique relation per pair (directional)
    UNIQUE (organization_id, target_organization_id)
);

CREATE INDEX idx_diplomatic_org ON organizations.diplomatic_relations(organization_id);
CREATE INDEX idx_diplomatic_target ON organizations.diplomatic_relations(target_organization_id);
CREATE INDEX idx_diplomatic_type ON organizations.diplomatic_relations(relation_type);

COMMENT ON TABLE organizations.diplomatic_relations IS 'Relations diplomatiques entre organisations';
COMMENT ON COLUMN organizations.diplomatic_relations.relation_type IS 'Type: allied, neutral, hostile, at_war, trade_agreement, non_aggression';
COMMENT ON COLUMN organizations.diplomatic_relations.expires_at IS 'Date d''expiration (NULL = permanent)';

-- ============================================================================
-- TRIGGER FUNCTIONS
-- ============================================================================

-- Auto-update population count when members change
CREATE OR REPLACE FUNCTION organizations.update_organization_population()
RETURNS TRIGGER AS $$
DECLARE
    org_id BIGINT;
BEGIN
    -- Get organization_id from either NEW or OLD record
    IF TG_OP = 'DELETE' THEN
        org_id := OLD.organization_id;
    ELSE
        org_id := NEW.organization_id;
    END IF;

    -- Update population count
    UPDATE organizations.organizations
    SET population = (
        SELECT COUNT(*)
        FROM organizations.members
        WHERE organization_id = org_id
        AND membership_status = 'active'
    )
    WHERE id = org_id;

    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    ELSE
        RETURN NEW;
    END IF;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_population_insert
    AFTER INSERT ON organizations.members
    FOR EACH ROW
    EXECUTE FUNCTION organizations.update_organization_population();

CREATE TRIGGER trigger_update_population_update
    AFTER UPDATE ON organizations.members
    FOR EACH ROW
    WHEN (OLD.membership_status IS DISTINCT FROM NEW.membership_status)
    EXECUTE FUNCTION organizations.update_organization_population();

CREATE TRIGGER trigger_update_population_delete
    AFTER DELETE ON organizations.members
    FOR EACH ROW
    EXECUTE FUNCTION organizations.update_organization_population();

-- Validate officer role compatibility
CREATE OR REPLACE FUNCTION organizations.validate_officer_role()
RETURNS TRIGGER AS $$
DECLARE
    org_type_id SMALLINT;
    is_compatible BOOLEAN;
    is_leader_role BOOLEAN;
    current_leader_count INT;
    is_member BOOLEAN;
    is_leader BOOLEAN;
BEGIN
    -- Get organization type
    SELECT organization_type_id INTO org_type_id
    FROM organizations.organizations
    WHERE id = NEW.organization_id;

    -- Check if unit is a member or the leader
    SELECT EXISTS (
        SELECT 1 FROM organizations.members
        WHERE organization_id = NEW.organization_id
        AND unit_id = NEW.unit_id
        AND membership_status = 'active'
    ) INTO is_member;

    SELECT EXISTS (
        SELECT 1 FROM organizations.organizations
        WHERE id = NEW.organization_id
        AND leader_unit_id = NEW.unit_id
    ) INTO is_leader;

    IF NOT (is_member OR is_leader) THEN
        RAISE EXCEPTION 'Unit must be a member or leader of the organization before becoming an officer';
    END IF;

    -- Check if role is compatible with organization type
    SELECT
        COUNT(*) > 0,
        COALESCE(MAX(orc.is_leader_role), false)
    INTO is_compatible, is_leader_role
    FROM organizations.organization_role_compatibility orc
    WHERE orc.organization_type_id = org_type_id
    AND orc.role_type_id = NEW.role_type_id;

    IF NOT is_compatible THEN
        RAISE EXCEPTION 'Role type % is not compatible with this organization type', NEW.role_type_id;
    END IF;

    -- If this is a leader role, ensure there's only one leader
    IF is_leader_role THEN
        SELECT COUNT(*) INTO current_leader_count
        FROM organizations.officers o
        JOIN organizations.organization_role_compatibility orc
            ON orc.role_type_id = o.role_type_id
            AND orc.organization_type_id = org_type_id
        WHERE o.organization_id = NEW.organization_id
        AND orc.is_leader_role = true
        AND (TG_OP = 'INSERT' OR o.id != NEW.id);

        IF current_leader_count > 0 THEN
            RAISE EXCEPTION 'Organization already has a leader';
        END IF;

        -- Update the organization's leader_unit_id
        UPDATE organizations.organizations
        SET leader_unit_id = NEW.unit_id
        WHERE id = NEW.organization_id;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_validate_officer_role
    BEFORE INSERT OR UPDATE ON organizations.officers
    FOR EACH ROW
    EXECUTE FUNCTION organizations.validate_officer_role();

-- Auto-remove officer when member is removed
CREATE OR REPLACE FUNCTION organizations.remove_officer_on_member_removal()
RETURNS TRIGGER AS $$
BEGIN
    IF OLD.membership_status = 'active' AND NEW.membership_status != 'active' THEN
        DELETE FROM organizations.officers
        WHERE organization_id = NEW.organization_id
        AND unit_id = NEW.unit_id;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_remove_officer_on_suspension
    AFTER UPDATE ON organizations.members
    FOR EACH ROW
    WHEN (OLD.membership_status IS DISTINCT FROM NEW.membership_status)
    EXECUTE FUNCTION organizations.remove_officer_on_member_removal();

-- Clean up leader_unit_id when officer is removed
CREATE OR REPLACE FUNCTION organizations.cleanup_leader_on_officer_removal()
RETURNS TRIGGER AS $$
DECLARE
    org_type_id SMALLINT;
    was_leader_role BOOLEAN;
BEGIN
    -- Get organization type
    SELECT organization_type_id INTO org_type_id
    FROM organizations.organizations
    WHERE id = OLD.organization_id;

    -- Check if this was a leader role
    SELECT is_leader_role INTO was_leader_role
    FROM organizations.organization_role_compatibility
    WHERE organization_type_id = org_type_id
    AND role_type_id = OLD.role_type_id;

    -- If it was a leader role, clear the organization's leader_unit_id
    IF was_leader_role THEN
        UPDATE organizations.organizations
        SET leader_unit_id = NULL
        WHERE id = OLD.organization_id
        AND leader_unit_id = OLD.unit_id;
    END IF;

    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_cleanup_leader_on_removal
    BEFORE DELETE ON organizations.officers
    FOR EACH ROW
    EXECUTE FUNCTION organizations.cleanup_leader_on_officer_removal();
