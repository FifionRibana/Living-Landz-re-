-- Extend resources system with complete item management

-- Item types (déjà existants dans units, on les déplace ici)
CREATE TABLE IF NOT EXISTS resources.item_types (
    id SMALLINT PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL,
    archived BOOLEAN DEFAULT FALSE
);

INSERT INTO resources.item_types (id, name) VALUES
    (0, 'Unknown'),
    (1, 'Resource'),      -- Matière première (bois, pierre, minerai)
    (2, 'Consumable'),    -- Consommable (nourriture, potion)
    (3, 'Equipment'),     -- Équipement général
    (4, 'Tool'),          -- Outil (pioche, hache)
    (5, 'Weapon'),        -- Arme
    (6, 'Armor'),         -- Armure
    (7, 'Accessory')      -- Accessoire (anneau, collier)
ON CONFLICT (id) DO NOTHING;

-- Equipment slots (pour la cohérence avec units)
CREATE TABLE IF NOT EXISTS resources.equipment_slots (
    id SMALLINT PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL,
    archived BOOLEAN DEFAULT FALSE
);

INSERT INTO resources.equipment_slots (id, name) VALUES
    (0, 'Unknown'),
    (1, 'Head'),
    (2, 'Chest'),
    (3, 'Legs'),
    (4, 'Feet'),
    (5, 'Hands'),
    (6, 'MainHand'),
    (7, 'OffHand'),
    (8, 'Back'),
    (9, 'Neck'),
    (10, 'Ring1'),
    (11, 'Ring2')
ON CONFLICT (id) DO NOTHING;

-- Items (définitions d'items)
CREATE TABLE resources.items (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL UNIQUE,
    item_type_id SMALLINT NOT NULL REFERENCES resources.item_types(id),
    category_id SMALLINT REFERENCES resources.resource_categories(id),

    -- Propriétés physiques
    weight_kg DECIMAL(10, 3) NOT NULL DEFAULT 0.001,
    volume_liters DECIMAL(10, 3) NOT NULL DEFAULT 0.001,

    -- Économie
    base_price INT NOT NULL DEFAULT 0, -- Prix de base en pièces de monnaie

    -- Propriétés de périssabilité
    is_perishable BOOLEAN DEFAULT FALSE,
    base_decay_rate_per_day DECIMAL(5, 4) DEFAULT 0, -- Taux de dégradation par jour (0.0 - 1.0)

    -- Équipement
    is_equipable BOOLEAN DEFAULT FALSE,
    equipment_slot_id SMALLINT REFERENCES resources.equipment_slots(id),

    -- Craft
    is_craftable BOOLEAN DEFAULT FALSE,

    -- Description et média
    description TEXT,
    image_url VARCHAR(500),

    -- Méta
    archived BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_items_type ON resources.items(item_type_id);
CREATE INDEX idx_items_category ON resources.items(category_id);
CREATE INDEX idx_items_equipable ON resources.items(is_equipable) WHERE is_equipable = TRUE;
CREATE INDEX idx_items_craftable ON resources.items(is_craftable) WHERE is_craftable = TRUE;
CREATE INDEX idx_items_perishable ON resources.items(is_perishable) WHERE is_perishable = TRUE;

COMMENT ON TABLE resources.items IS 'Item definitions - base templates for all items in the game';
COMMENT ON COLUMN resources.items.base_price IS 'Base price in copper coins (100 copper = 1 silver, 100 silver = 1 gold)';
COMMENT ON COLUMN resources.items.base_decay_rate_per_day IS 'Base decay rate per day for perishable items (0.0 = no decay, 1.0 = complete decay in 1 day)';

-- Stat modifiers for items (bonus donnés par les items)
CREATE TABLE resources.item_stat_modifiers (
    item_id INT NOT NULL REFERENCES resources.items(id) ON DELETE CASCADE,
    stat_name VARCHAR NOT NULL, -- 'strength_bonus', 'defense_physical', 'inventory_capacity_kg', etc.
    modifier_value INT NOT NULL,
    PRIMARY KEY (item_id, stat_name)
);

CREATE INDEX idx_item_stat_modifiers ON resources.item_stat_modifiers(item_id);

COMMENT ON TABLE resources.item_stat_modifiers IS 'Stat bonuses provided by items when equipped or used';

-- Exemples d'items de base (on remplace ceux de units)
INSERT INTO resources.items
    (id, name, item_type_id, category_id, weight_kg, volume_liters, base_price, is_perishable, base_decay_rate_per_day, is_equipable, equipment_slot_id, is_craftable, description)
VALUES
    -- Resources (matières premières)
    (1, 'Wood', 1, 1, 5.0, 10.0, 2, FALSE, 0, FALSE, NULL, FALSE, 'Basic lumber resource'),
    (2, 'Stone', 1, 3, 10.0, 5.0, 1, FALSE, 0, FALSE, NULL, FALSE, 'Basic stone resource'),
    (3, 'Iron Ore', 1, 2, 8.0, 3.0, 5, FALSE, 0, FALSE, NULL, FALSE, 'Raw iron ore'),
    (4, 'Wheat', 1, 10, 1.0, 2.0, 3, FALSE, 0, FALSE, NULL, FALSE, 'Grain for bread'),
    (5, 'Coal', 1, 3, 3.0, 2.0, 4, FALSE, 0, FALSE, NULL, FALSE, 'Fuel for smelting'),

    -- Food (consommables périssables)
    (10, 'Bread', 2, 4, 0.5, 0.8, 5, TRUE, 0.1, FALSE, NULL, TRUE, 'Basic food item, lasts ~10 days'),
    (11, 'Apple', 2, 9, 0.2, 0.3, 2, TRUE, 0.15, FALSE, NULL, FALSE, 'Fresh fruit, lasts ~7 days'),
    (12, 'Meat', 2, 8, 1.0, 1.0, 8, TRUE, 0.3, FALSE, NULL, FALSE, 'Fresh meat, lasts ~3 days'),
    (13, 'Cooked Meat', 2, 8, 0.8, 0.8, 12, TRUE, 0.2, FALSE, NULL, TRUE, 'Cooked meat, lasts ~5 days'),

    -- Tools
    (20, 'Iron Pickaxe', 4, 6, 3.0, 2.0, 50, FALSE, 0, TRUE, 6, TRUE, 'Tool for mining'),
    (21, 'Iron Axe', 4, 6, 2.5, 1.8, 45, FALSE, 0, TRUE, 6, TRUE, 'Tool for lumberjacking'),
    (22, 'Fishing Rod', 4, 6, 1.5, 3.0, 30, FALSE, 0, TRUE, 6, TRUE, 'Tool for fishing'),
    (23, 'Hammer', 4, 6, 2.0, 1.5, 35, FALSE, 0, TRUE, 6, TRUE, 'Tool for blacksmithing'),

    -- Weapons
    (30, 'Iron Sword', 5, 6, 3.5, 1.2, 100, FALSE, 0, TRUE, 6, TRUE, 'Basic melee weapon'),
    (31, 'Wooden Bow', 5, 1, 1.0, 2.5, 60, FALSE, 0, TRUE, 6, TRUE, 'Basic ranged weapon'),
    (32, 'Iron Shield', 5, 6, 5.0, 4.0, 80, FALSE, 0, TRUE, 7, TRUE, 'Basic shield'),

    -- Armor
    (40, 'Leather Helmet', 6, 6, 0.5, 1.0, 40, FALSE, 0, TRUE, 1, TRUE, 'Light head protection'),
    (41, 'Leather Chest', 6, 6, 2.0, 3.0, 80, FALSE, 0, TRUE, 2, TRUE, 'Light body protection'),
    (42, 'Leather Pants', 6, 6, 1.5, 2.5, 60, FALSE, 0, TRUE, 3, TRUE, 'Light leg protection'),
    (43, 'Leather Boots', 6, 6, 0.8, 1.5, 30, FALSE, 0, TRUE, 4, TRUE, 'Light foot protection'),

    -- Equipment (backpacks, etc.)
    (50, 'Small Backpack', 3, 6, 1.0, 5.0, 25, FALSE, 0, TRUE, 8, TRUE, 'Increases carrying capacity'),
    (51, 'Large Backpack', 3, 6, 2.5, 10.0, 60, FALSE, 0, TRUE, 8, TRUE, 'Greatly increases carrying capacity');

-- Stats des items (bonus)
INSERT INTO resources.item_stat_modifiers (item_id, stat_name, modifier_value) VALUES
    -- Pickaxe
    (20, 'mining_bonus', 20),

    -- Axe
    (21, 'lumberjacking_bonus', 20),

    -- Fishing Rod
    (22, 'fishing_bonus', 20),

    -- Hammer
    (23, 'blacksmithing_bonus', 20),

    -- Sword
    (30, 'melee_attack_bonus', 10),
    (30, 'strength_bonus', 2),

    -- Bow
    (31, 'ranged_attack_bonus', 10),

    -- Shield
    (32, 'defense_physical', 15),
    (32, 'defense_melee', 20),

    -- Armor
    (40, 'defense_physical', 3),
    (41, 'defense_physical', 8),
    (42, 'defense_physical', 6),
    (43, 'defense_physical', 4),

    -- Backpacks
    (50, 'inventory_capacity_kg', 20),
    (51, 'inventory_capacity_kg', 50);

-- Mettre à jour la séquence
SELECT setval('resources.items_id_seq', GREATEST(100, (SELECT MAX(id) FROM resources.items)));
