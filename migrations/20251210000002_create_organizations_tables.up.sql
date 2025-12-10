-- Migration: Create organizations main tables
-- Description: Core tables for organization data, treasury, and headquarters

-- ============================================================================
-- MAIN ORGANIZATIONS TABLE
-- ============================================================================

CREATE TABLE organizations.organizations (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    organization_type_id SMALLINT NOT NULL REFERENCES organizations.organization_types(id),

    -- Hierarchy/vassalité
    parent_organization_id BIGINT REFERENCES organizations.organizations(id) ON DELETE SET NULL,

    -- Headquarters location (siege principal)
    headquarters_cell_q INT,
    headquarters_cell_r INT,

    -- Territory (calculé automatiquement)
    total_area_km2 DECIMAL(10, 2) DEFAULT 0,

    -- Economy
    treasury_gold INT DEFAULT 0 CHECK (treasury_gold >= 0),

    -- Leadership
    leader_unit_id BIGINT REFERENCES units.units(id) ON DELETE SET NULL,

    -- Identity
    emblem_url VARCHAR(255),

    -- Population (calculée automatiquement)
    population INT DEFAULT 0 CHECK (population >= 0),

    -- Metadata
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    -- Constraints
    CONSTRAINT valid_headquarters CHECK (
        (headquarters_cell_q IS NULL AND headquarters_cell_r IS NULL) OR
        (headquarters_cell_q IS NOT NULL AND headquarters_cell_r IS NOT NULL)
    )
);

-- Indexes
CREATE INDEX idx_organizations_type ON organizations.organizations(organization_type_id);
CREATE INDEX idx_organizations_parent ON organizations.organizations(parent_organization_id);
CREATE INDEX idx_organizations_leader ON organizations.organizations(leader_unit_id);
CREATE INDEX idx_organizations_headquarters ON organizations.organizations(headquarters_cell_q, headquarters_cell_r);
CREATE INDEX idx_organizations_name ON organizations.organizations(name);

-- Comments
COMMENT ON TABLE organizations.organizations IS 'Table principale des organisations (villes, royaumes, guildes, etc.)';
COMMENT ON COLUMN organizations.organizations.parent_organization_id IS 'Organisation parente (vassalité)';
COMMENT ON COLUMN organizations.organizations.headquarters_cell_q IS 'Coordonnée Q de la cellule du siège';
COMMENT ON COLUMN organizations.organizations.total_area_km2 IS 'Superficie totale en km² (calculée automatiquement)';
COMMENT ON COLUMN organizations.organizations.treasury_gold IS 'Or dans la trésorerie';
COMMENT ON COLUMN organizations.organizations.population IS 'Population totale (calculée automatiquement)';

-- ============================================================================
-- TERRITORY CELLS
-- ============================================================================

CREATE TABLE organizations.territory_cells (
    organization_id BIGINT NOT NULL REFERENCES organizations.organizations(id) ON DELETE CASCADE,
    cell_q INT NOT NULL,
    cell_r INT NOT NULL,
    claimed_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (organization_id, cell_q, cell_r),

    -- Ensure a cell can only belong to one organization
    UNIQUE (cell_q, cell_r)
);

CREATE INDEX idx_territory_cells_location ON organizations.territory_cells(cell_q, cell_r);
CREATE INDEX idx_territory_cells_org ON organizations.territory_cells(organization_id);

COMMENT ON TABLE organizations.territory_cells IS 'Cellules de territoire contrôlées par les organisations';
COMMENT ON COLUMN organizations.territory_cells.claimed_at IS 'Date de revendication du territoire';

-- ============================================================================
-- TREASURY ITEMS (Inventory)
-- ============================================================================

CREATE TABLE organizations.treasury_items (
    id BIGSERIAL PRIMARY KEY,
    organization_id BIGINT NOT NULL REFERENCES organizations.organizations(id) ON DELETE CASCADE,
    item_instance_id BIGINT NOT NULL REFERENCES resources.item_instances(id) ON DELETE CASCADE,
    quantity INT NOT NULL DEFAULT 1 CHECK (quantity > 0),
    stored_at TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE (organization_id, item_instance_id)
);

CREATE INDEX idx_treasury_items_org ON organizations.treasury_items(organization_id);
CREATE INDEX idx_treasury_items_instance ON organizations.treasury_items(item_instance_id);

COMMENT ON TABLE organizations.treasury_items IS 'Inventaire/trésorerie des organisations';
COMMENT ON COLUMN organizations.treasury_items.quantity IS 'Quantité d''items (pour items stackables)';

-- ============================================================================
-- TRIGGER FUNCTIONS
-- ============================================================================

-- Auto-update updated_at timestamp
CREATE OR REPLACE FUNCTION organizations.update_organization_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_organization_timestamp
    BEFORE UPDATE ON organizations.organizations
    FOR EACH ROW
    EXECUTE FUNCTION organizations.update_organization_timestamp();

-- Auto-calculate territory area
-- Assumes each cell is approximately 0.01 km² (100m x 100m)
CREATE OR REPLACE FUNCTION organizations.update_territory_area()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE organizations.organizations
    SET total_area_km2 = (
        SELECT COUNT(*) * 0.01
        FROM organizations.territory_cells
        WHERE organization_id = NEW.organization_id
    )
    WHERE id = NEW.organization_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_territory_area_insert
    AFTER INSERT ON organizations.territory_cells
    FOR EACH ROW
    EXECUTE FUNCTION organizations.update_territory_area();

CREATE TRIGGER trigger_update_territory_area_delete
    AFTER DELETE ON organizations.territory_cells
    FOR EACH ROW
    EXECUTE FUNCTION organizations.update_territory_area();

-- ============================================================================
-- VALIDATION FUNCTION
-- ============================================================================

-- Validate organization constraints based on type
CREATE OR REPLACE FUNCTION organizations.validate_organization_constraints()
RETURNS TRIGGER AS $$
DECLARE
    org_type RECORD;
    territory_count INT;
    current_population INT;
    current_area DECIMAL;
BEGIN
    -- Get organization type info
    SELECT * INTO org_type
    FROM organizations.organization_types
    WHERE id = NEW.organization_type_id;

    -- Check if requires territory
    IF org_type.requires_territory THEN
        SELECT COUNT(*) INTO territory_count
        FROM organizations.territory_cells
        WHERE organization_id = NEW.id;

        -- Non-territorial organizations must have headquarters in existing org
        IF territory_count = 0 AND NOT org_type.category = 'territorial' THEN
            IF NEW.headquarters_cell_q IS NULL THEN
                RAISE EXCEPTION 'Organization type % requires a headquarters location', org_type.name;
            END IF;
        END IF;
    END IF;

    -- Validate parent relationship
    IF NEW.parent_organization_id IS NOT NULL THEN
        IF NOT org_type.can_have_parent THEN
            RAISE EXCEPTION 'Organization type % cannot have a parent organization', org_type.name;
        END IF;
    END IF;

    -- Check minimum population (if updating)
    IF TG_OP = 'UPDATE' AND org_type.min_population IS NOT NULL THEN
        IF NEW.population < org_type.min_population THEN
            RAISE EXCEPTION 'Organization type % requires minimum population of %, current: %',
                org_type.name, org_type.min_population, NEW.population;
        END IF;
    END IF;

    -- Check minimum area (if updating)
    IF TG_OP = 'UPDATE' AND org_type.min_area_km2 IS NOT NULL THEN
        IF NEW.total_area_km2 < org_type.min_area_km2 THEN
            RAISE EXCEPTION 'Organization type % requires minimum area of % km², current: % km²',
                org_type.name, org_type.min_area_km2, NEW.total_area_km2;
        END IF;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_validate_organization
    BEFORE INSERT OR UPDATE ON organizations.organizations
    FOR EACH ROW
    EXECUTE FUNCTION organizations.validate_organization_constraints();
