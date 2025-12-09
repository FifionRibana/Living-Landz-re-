-- Create units main tables

-- Main units table
CREATE TABLE units.units (
    id BIGSERIAL PRIMARY KEY,
    player_id BIGINT NULL, -- NULL si c'est un NPC, sinon ID du joueur
    first_name VARCHAR(100) NOT NULL,
    last_name VARCHAR(100) NOT NULL,
    level INT NOT NULL DEFAULT 1,
    avatar_url VARCHAR(500),

    -- Position actuelle
    current_cell_q INT NOT NULL,
    current_cell_r INT NOT NULL,
    current_chunk_x INT NOT NULL,
    current_chunk_y INT NOT NULL,

    -- Profession
    profession_id SMALLINT NOT NULL REFERENCES units.professions(id) DEFAULT 0,

    -- Argent
    money BIGINT NOT NULL DEFAULT 0,

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_units_player ON units.units(player_id) WHERE player_id IS NOT NULL;
CREATE INDEX idx_units_position ON units.units(current_chunk_x, current_chunk_y, current_cell_q, current_cell_r);
CREATE INDEX idx_units_profession ON units.units(profession_id);

COMMENT ON TABLE units.units IS 'Main table for all units (players and NPCs)';
COMMENT ON COLUMN units.units.player_id IS 'NULL for NPCs, player ID for player characters';

-- Base stats (statistiques principales)
CREATE TABLE units.unit_base_stats (
    unit_id BIGINT PRIMARY KEY REFERENCES units.units(id) ON DELETE CASCADE,

    -- Statistiques principales
    strength INT NOT NULL DEFAULT 10,      -- Force
    agility INT NOT NULL DEFAULT 10,       -- Agilité
    constitution INT NOT NULL DEFAULT 10,  -- Constitution
    intelligence INT NOT NULL DEFAULT 10,  -- Intelligence
    wisdom INT NOT NULL DEFAULT 10,        -- Sagesse
    charisma INT NOT NULL DEFAULT 10,      -- Charisme

    updated_at TIMESTAMPTZ DEFAULT NOW()
);

COMMENT ON TABLE units.unit_base_stats IS 'Primary stats for units - stored in DB';

-- Derived stats (statistiques dérivées)
CREATE TABLE units.unit_derived_stats (
    unit_id BIGINT PRIMARY KEY REFERENCES units.units(id) ON DELETE CASCADE,

    -- Points de vie
    max_hp INT NOT NULL DEFAULT 100,
    current_hp INT NOT NULL DEFAULT 100,

    -- Mental stats
    happiness INT NOT NULL DEFAULT 50,      -- 0-100
    mental_health INT NOT NULL DEFAULT 100, -- 0-100

    -- Capacités (peuvent être recalculées mais on les stocke pour performance)
    base_inventory_capacity_kg INT NOT NULL DEFAULT 50,
    current_weight_kg DECIMAL(10, 3) NOT NULL DEFAULT 0,

    updated_at TIMESTAMPTZ DEFAULT NOW(),

    CONSTRAINT chk_happiness CHECK (happiness >= 0 AND happiness <= 100),
    CONSTRAINT chk_mental_health CHECK (mental_health >= 0 AND mental_health <= 100),
    CONSTRAINT chk_current_hp CHECK (current_hp >= 0 AND current_hp <= max_hp)
);

COMMENT ON TABLE units.unit_derived_stats IS 'Derived stats - some can be calculated but stored for performance';
COMMENT ON COLUMN units.unit_derived_stats.base_inventory_capacity_kg IS 'Base capacity, actual capacity includes equipment bonuses';

-- Unit skills (compétences par unité)
CREATE TABLE units.unit_skills (
    unit_id BIGINT NOT NULL REFERENCES units.units(id) ON DELETE CASCADE,
    skill_id SMALLINT NOT NULL REFERENCES units.skills(id),
    xp BIGINT NOT NULL DEFAULT 0,
    level INT NOT NULL DEFAULT 1,
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (unit_id, skill_id)
);

CREATE INDEX idx_unit_skills_unit ON units.unit_skills(unit_id);

COMMENT ON TABLE units.unit_skills IS 'Skills and experience for each unit';

-- Unit inventory (inventaire)
CREATE TABLE units.unit_inventory (
    id BIGSERIAL PRIMARY KEY,
    unit_id BIGINT NOT NULL REFERENCES units.units(id) ON DELETE CASCADE,
    item_id INT NOT NULL REFERENCES units.items(id),
    quantity INT NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(unit_id, item_id),
    CONSTRAINT chk_quantity CHECK (quantity > 0)
);

CREATE INDEX idx_unit_inventory_unit ON units.unit_inventory(unit_id);
CREATE INDEX idx_unit_inventory_item ON units.unit_inventory(item_id);

COMMENT ON TABLE units.unit_inventory IS 'Items owned by each unit';

-- Unit equipment (équipement équipé)
CREATE TABLE units.unit_equipment (
    unit_id BIGINT NOT NULL REFERENCES units.units(id) ON DELETE CASCADE,
    equipment_slot_id SMALLINT NOT NULL REFERENCES units.equipment_slots(id),
    item_id INT NOT NULL REFERENCES units.items(id),
    equipped_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (unit_id, equipment_slot_id)
);

CREATE INDEX idx_unit_equipment_unit ON units.unit_equipment(unit_id);

COMMENT ON TABLE units.unit_equipment IS 'Currently equipped items for each unit';

-- Automated actions (actions automatisables)
CREATE TABLE units.unit_automated_actions (
    id BIGSERIAL PRIMARY KEY,
    unit_id BIGINT NOT NULL REFERENCES units.units(id) ON DELETE CASCADE,
    action_type VARCHAR(100) NOT NULL, -- Ex: 'auto_craft_bread', 'auto_harvest_wheat'
    is_enabled BOOLEAN DEFAULT TRUE,
    parameters JSONB, -- Paramètres de configuration de l'action
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_unit_automated_actions_unit ON units.unit_automated_actions(unit_id);
CREATE INDEX idx_unit_automated_actions_enabled ON units.unit_automated_actions(unit_id, is_enabled)
    WHERE is_enabled = TRUE;

COMMENT ON TABLE units.unit_automated_actions IS 'Automated actions configured for each unit';
COMMENT ON COLUMN units.unit_automated_actions.parameters IS 'JSON configuration for the action';

-- Consumption demands (demandes de consommation)
CREATE TABLE units.unit_consumption_demands (
    unit_id BIGINT NOT NULL REFERENCES units.units(id) ON DELETE CASCADE,
    item_id INT NOT NULL REFERENCES units.items(id),
    quantity_per_day DECIMAL(10, 3) NOT NULL,
    priority INT NOT NULL DEFAULT 5, -- 1 (highest) to 10 (lowest)
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (unit_id, item_id),
    CONSTRAINT chk_quantity_per_day CHECK (quantity_per_day > 0),
    CONSTRAINT chk_priority CHECK (priority >= 1 AND priority <= 10)
);

CREATE INDEX idx_unit_consumption_unit ON units.unit_consumption_demands(unit_id);

COMMENT ON TABLE units.unit_consumption_demands IS 'Daily consumption needs for each unit';
COMMENT ON COLUMN units.unit_consumption_demands.priority IS 'Priority 1-10, where 1 is highest priority';

-- Fonction pour mettre à jour updated_at automatiquement
CREATE OR REPLACE FUNCTION units.update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Triggers pour updated_at
CREATE TRIGGER update_units_updated_at BEFORE UPDATE ON units.units
    FOR EACH ROW EXECUTE FUNCTION units.update_updated_at_column();

CREATE TRIGGER update_unit_base_stats_updated_at BEFORE UPDATE ON units.unit_base_stats
    FOR EACH ROW EXECUTE FUNCTION units.update_updated_at_column();

CREATE TRIGGER update_unit_derived_stats_updated_at BEFORE UPDATE ON units.unit_derived_stats
    FOR EACH ROW EXECUTE FUNCTION units.update_updated_at_column();

CREATE TRIGGER update_unit_skills_updated_at BEFORE UPDATE ON units.unit_skills
    FOR EACH ROW EXECUTE FUNCTION units.update_updated_at_column();

CREATE TRIGGER update_unit_inventory_updated_at BEFORE UPDATE ON units.unit_inventory
    FOR EACH ROW EXECUTE FUNCTION units.update_updated_at_column();

CREATE TRIGGER update_unit_automated_actions_updated_at BEFORE UPDATE ON units.unit_automated_actions
    FOR EACH ROW EXECUTE FUNCTION units.update_updated_at_column();
