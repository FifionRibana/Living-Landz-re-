-- Migration: Create tables and slug columns required by the game seed system
-- Run this ONCE before using game-seed for the first time.

-- ═══════════════════════════════════════════════════════════
-- Slug columns on existing tables
-- ═══════════════════════════════════════════════════════════

ALTER TABLE resources.resource_categories
    ADD COLUMN IF NOT EXISTS slug VARCHAR(64) UNIQUE;
ALTER TABLE resources.resource_specific_types
    ADD COLUMN IF NOT EXISTS slug VARCHAR(64) UNIQUE;
ALTER TABLE buildings.building_categories
    ADD COLUMN IF NOT EXISTS slug VARCHAR(64) UNIQUE;
ALTER TABLE buildings.building_specific_types
    ADD COLUMN IF NOT EXISTS slug VARCHAR(64) UNIQUE;
ALTER TABLE resources.item_types
    ADD COLUMN IF NOT EXISTS slug VARCHAR(64) UNIQUE;
ALTER TABLE resources.equipment_slots
    ADD COLUMN IF NOT EXISTS slug VARCHAR(64) UNIQUE;
ALTER TABLE units.professions
    ADD COLUMN IF NOT EXISTS slug VARCHAR(64) UNIQUE;
ALTER TABLE units.skills
    ADD COLUMN IF NOT EXISTS slug VARCHAR(64) UNIQUE;
ALTER TABLE resources.items
    ADD COLUMN IF NOT EXISTS slug VARCHAR(64) UNIQUE;
ALTER TABLE resources.recipes
    ADD COLUMN IF NOT EXISTS slug VARCHAR(64) UNIQUE;
ALTER TABLE buildings.building_types
    ADD COLUMN IF NOT EXISTS slug VARCHAR(64) UNIQUE;

-- ═══════════════════════════════════════════════════════════
-- Translations
-- ═══════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS game.translations (
    entity_type  VARCHAR(50) NOT NULL,
    entity_id    INT NOT NULL,
    language_id  SMALLINT NOT NULL REFERENCES game.languages(id),
    field        VARCHAR(50) NOT NULL,
    value        TEXT NOT NULL,
    PRIMARY KEY (entity_type, entity_id, language_id, field)
);

CREATE INDEX IF NOT EXISTS idx_translations_entity
    ON game.translations(entity_type, entity_id);

-- ═══════════════════════════════════════════════════════════
-- Construction duration on building types
-- ═══════════════════════════════════════════════════════════

ALTER TABLE buildings.building_types
    ADD COLUMN IF NOT EXISTS construction_duration_seconds INT NOT NULL DEFAULT 15;

-- ═══════════════════════════════════════════════════════════
-- Construction costs
-- ═══════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS buildings.construction_costs (
    building_type_id INT NOT NULL
        REFERENCES buildings.building_types(id) ON DELETE CASCADE,
    item_id INT NOT NULL REFERENCES resources.items(id),
    quantity INT NOT NULL,
    PRIMARY KEY (building_type_id, item_id),
    CONSTRAINT chk_construction_cost_qty CHECK (quantity > 0)
);

-- ═══════════════════════════════════════════════════════════
-- Harvest yields
-- ═══════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS resources.harvest_yields (
    id                        SERIAL PRIMARY KEY,
    resource_specific_type_id SMALLINT NOT NULL
        REFERENCES resources.resource_specific_types(id),
    result_item_id            INT NOT NULL REFERENCES resources.items(id),
    base_quantity             INT NOT NULL DEFAULT 1,
    quality_min               DECIMAL(3,2) NOT NULL DEFAULT 0.50,
    quality_max               DECIMAL(3,2) NOT NULL DEFAULT 1.00,
    required_profession_id    SMALLINT,
    required_tool_item_id     INT REFERENCES resources.items(id),
    tool_bonus_quantity       INT DEFAULT 0,
    duration_seconds          INT NOT NULL DEFAULT 30,
    CONSTRAINT chk_harvest_base_qty CHECK (base_quantity > 0),
    CONSTRAINT chk_harvest_quality  CHECK (quality_min <= quality_max)
);
