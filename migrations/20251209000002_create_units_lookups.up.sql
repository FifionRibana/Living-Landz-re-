-- Create units lookup tables

-- Professions (métiers)
CREATE TABLE units.professions (
    id SMALLINT PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL,
    description TEXT,
    base_inventory_capacity_bonus INT DEFAULT 0, -- Bonus en kg
    archived BOOLEAN DEFAULT FALSE
);

INSERT INTO units.professions (id, name, description, base_inventory_capacity_bonus) VALUES
    (0, 'Unknown', 'No profession assigned', 0),
    (1, 'Baker', 'Specialized in baking bread and pastries', 5),
    (2, 'Farmer', 'Grows crops and manages agricultural land', 10),
    (3, 'Warrior', 'Trained in combat and military tactics', 15),
    (4, 'Blacksmith', 'Crafts metal tools and weapons', 20),
    (5, 'Carpenter', 'Works with wood to create structures and furniture', 15),
    (6, 'Miner', 'Extracts minerals and ores from the earth', 25),
    (7, 'Merchant', 'Trades goods and manages commerce', 30),
    (8, 'Hunter', 'Tracks and hunts animals for food and resources', 12),
    (9, 'Healer', 'Provides medical care and healing', 8),
    (10, 'Scholar', 'Studies and researches various subjects', 5),
    (11, 'Cook', 'Prepares meals and manages kitchens', 10),
    (12, 'Fisherman', 'Catches fish from rivers and oceans', 15),
    (13, 'Lumberjack', 'Cuts down trees and processes timber', 20),
    (14, 'Mason', 'Builds with stone and creates masonry structures', 15),
    (15, 'Brewer', 'Produces alcoholic beverages', 12);

-- Skills (compétences)
CREATE TABLE units.skills (
    id SMALLINT PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL,
    description TEXT,
    primary_stat VARCHAR NOT NULL, -- strength, agility, constitution, intelligence, wisdom, charisma
    archived BOOLEAN DEFAULT FALSE
);

INSERT INTO units.skills (id, name, description, primary_stat) VALUES
    -- Force-based skills
    (1, 'MeleeAttack', 'Close combat effectiveness', 'strength'),
    (2, 'Carrying', 'Ability to carry heavy loads', 'strength'),
    (3, 'Mining', 'Efficiency at extracting minerals', 'strength'),
    (4, 'Lumberjacking', 'Tree cutting and timber processing', 'strength'),
    (5, 'Blacksmithing', 'Metalworking and tool crafting', 'strength'),

    -- Agility-based skills
    (10, 'RangedAttack', 'Accuracy with bows and ranged weapons', 'agility'),
    (11, 'Dodging', 'Evasion and avoiding attacks', 'agility'),
    (12, 'Stealth', 'Moving silently and unseen', 'agility'),
    (13, 'Fishing', 'Catching fish efficiently', 'agility'),
    (14, 'Hunting', 'Tracking and hunting animals', 'agility'),

    -- Constitution-based skills
    (20, 'Endurance', 'Stamina and resistance to fatigue', 'constitution'),
    (21, 'DiseaseResistance', 'Resistance to illness', 'constitution'),
    (22, 'PoisonResistance', 'Resistance to toxins', 'constitution'),

    -- Intelligence-based skills
    (30, 'Crafting', 'General crafting ability', 'intelligence'),
    (31, 'Engineering', 'Building complex structures', 'intelligence'),
    (32, 'Alchemy', 'Creating potions and compounds', 'intelligence'),
    (33, 'Research', 'Learning and discovering new knowledge', 'intelligence'),

    -- Wisdom-based skills
    (40, 'Farming', 'Growing crops efficiently', 'wisdom'),
    (41, 'Cooking', 'Preparing quality meals', 'wisdom'),
    (42, 'Baking', 'Creating bread and pastries', 'wisdom'),
    (43, 'Healing', 'Medical treatment and care', 'wisdom'),
    (44, 'AnimalHandling', 'Managing and training animals', 'wisdom'),
    (45, 'Brewing', 'Creating alcoholic beverages', 'wisdom'),

    -- Charisma-based skills
    (50, 'Trading', 'Getting better prices in commerce', 'charisma'),
    (51, 'Leadership', 'Inspiring and commanding others', 'charisma'),
    (52, 'Persuasion', 'Convincing others', 'charisma'),
    (53, 'Negotiation', 'Reaching favorable agreements', 'charisma');

-- Profession skill bonuses (bonus par métier)
CREATE TABLE units.profession_skill_bonuses (
    profession_id SMALLINT NOT NULL REFERENCES units.professions(id),
    skill_id SMALLINT NOT NULL REFERENCES units.skills(id),
    bonus_percentage INT NOT NULL, -- Bonus en pourcentage (ex: 20 = +20%)
    PRIMARY KEY (profession_id, skill_id)
);

-- Définir les bonus de skills pour chaque profession
INSERT INTO units.profession_skill_bonuses (profession_id, skill_id, bonus_percentage) VALUES
    -- Baker
    (1, 42, 30), -- Baking +30%
    (1, 41, 15), -- Cooking +15%
    (1, 50, 10), -- Trading +10%

    -- Farmer
    (2, 40, 30), -- Farming +30%
    (2, 44, 15), -- AnimalHandling +15%
    (2, 2, 10),  -- Carrying +10%

    -- Warrior
    (3, 1, 30),  -- MeleeAttack +30%
    (3, 11, 20), -- Dodging +20%
    (3, 20, 15), -- Endurance +15%

    -- Blacksmith
    (4, 5, 35),  -- Blacksmithing +35%
    (4, 30, 15), -- Crafting +15%
    (4, 3, 10),  -- Mining +10%

    -- Carpenter
    (5, 30, 25), -- Crafting +25%
    (5, 4, 20),  -- Lumberjacking +20%
    (5, 31, 15), -- Engineering +15%

    -- Miner
    (6, 3, 35),  -- Mining +35%
    (6, 2, 20),  -- Carrying +20%
    (6, 20, 15), -- Endurance +15%

    -- Merchant
    (7, 50, 35), -- Trading +35%
    (7, 52, 20), -- Persuasion +20%
    (7, 53, 20), -- Negotiation +20%

    -- Hunter
    (8, 14, 30), -- Hunting +30%
    (8, 10, 25), -- RangedAttack +25%
    (8, 12, 15), -- Stealth +15%

    -- Healer
    (9, 43, 35), -- Healing +35%
    (9, 32, 20), -- Alchemy +20%
    (9, 33, 10), -- Research +10%

    -- Scholar
    (10, 33, 30), -- Research +30%
    (10, 32, 15), -- Alchemy +15%
    (10, 31, 15), -- Engineering +15%

    -- Cook
    (11, 41, 35), -- Cooking +35%
    (11, 42, 15), -- Baking +15%
    (11, 50, 10), -- Trading +10%

    -- Fisherman
    (12, 13, 35), -- Fishing +35%
    (12, 14, 10), -- Hunting +10%
    (12, 2, 15),  -- Carrying +15%

    -- Lumberjack
    (13, 4, 35),  -- Lumberjacking +35%
    (13, 2, 20),  -- Carrying +20%
    (13, 20, 15), -- Endurance +15%

    -- Mason
    (14, 31, 30), -- Engineering +30%
    (14, 30, 20), -- Crafting +20%
    (14, 3, 10),  -- Mining +10%

    -- Brewer
    (15, 45, 35), -- Brewing +35%
    (15, 41, 15), -- Cooking +15%
    (15, 50, 10); -- Trading +10%

-- Item types
CREATE TABLE units.item_types (
    id SMALLINT PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL,
    archived BOOLEAN DEFAULT FALSE
);

INSERT INTO units.item_types (id, name) VALUES
    (0, 'Unknown'),
    (1, 'Resource'),
    (2, 'Consumable'),
    (3, 'Equipment'),
    (4, 'Tool'),
    (5, 'Weapon'),
    (6, 'Armor'),
    (7, 'Accessory');

-- Equipment slots
CREATE TABLE units.equipment_slots (
    id SMALLINT PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL,
    archived BOOLEAN DEFAULT FALSE
);

INSERT INTO units.equipment_slots (id, name) VALUES
    (0, 'Unknown'),
    (1, 'Head'),
    (2, 'Chest'),
    (3, 'Legs'),
    (4, 'Feet'),
    (5, 'Hands'),
    (6, 'MainHand'),    -- Weapon or tool
    (7, 'OffHand'),     -- Shield or secondary weapon
    (8, 'Back'),        -- Backpack
    (9, 'Neck'),        -- Necklace
    (10, 'Ring1'),
    (11, 'Ring2');

-- Items (objets et ressources)
CREATE TABLE units.items (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    item_type_id SMALLINT NOT NULL REFERENCES units.item_types(id),
    description TEXT,
    weight_kg DECIMAL(10, 3) NOT NULL DEFAULT 0.001,
    is_equipable BOOLEAN DEFAULT FALSE,
    equipment_slot_id SMALLINT REFERENCES units.equipment_slots(id),
    archived BOOLEAN DEFAULT FALSE,
    UNIQUE(name, item_type_id)
);

CREATE INDEX idx_items_type ON units.items(item_type_id);
CREATE INDEX idx_items_equipable ON units.items(is_equipable) WHERE is_equipable = TRUE;

-- Stat modifiers for items (bonus donnés par les équipements)
CREATE TABLE units.item_stat_modifiers (
    item_id INT NOT NULL REFERENCES units.items(id) ON DELETE CASCADE,
    stat_name VARCHAR NOT NULL, -- 'strength_bonus', 'defense_physical', 'inventory_capacity_kg', etc.
    modifier_value INT NOT NULL,
    PRIMARY KEY (item_id, stat_name)
);

CREATE INDEX idx_item_stat_modifiers ON units.item_stat_modifiers(item_id);

-- Exemples d'items de base
INSERT INTO units.items (id, name, item_type_id, description, weight_kg, is_equipable, equipment_slot_id) VALUES
    -- Resources
    (1, 'Wood', 1, 'Basic lumber resource', 5.0, FALSE, NULL),
    (2, 'Stone', 1, 'Basic stone resource', 10.0, FALSE, NULL),
    (3, 'Iron Ore', 1, 'Raw iron ore', 8.0, FALSE, NULL),
    (4, 'Wheat', 1, 'Grain for bread', 1.0, FALSE, NULL),
    (5, 'Bread', 2, 'Basic food item', 0.5, FALSE, NULL),

    -- Tools
    (10, 'Iron Pickaxe', 4, 'Tool for mining', 3.0, TRUE, 6),
    (11, 'Iron Axe', 4, 'Tool for lumberjacking', 2.5, TRUE, 6),
    (12, 'Fishing Rod', 4, 'Tool for fishing', 1.5, TRUE, 6),
    (13, 'Hammer', 4, 'Tool for blacksmithing', 2.0, TRUE, 6),

    -- Weapons
    (20, 'Iron Sword', 5, 'Basic melee weapon', 3.5, TRUE, 6),
    (21, 'Wooden Bow', 5, 'Basic ranged weapon', 1.0, TRUE, 6),
    (22, 'Iron Shield', 5, 'Basic shield', 5.0, TRUE, 7),

    -- Armor
    (30, 'Leather Helmet', 6, 'Light head protection', 0.5, TRUE, 1),
    (31, 'Leather Chest', 6, 'Light body protection', 2.0, TRUE, 2),
    (32, 'Leather Pants', 6, 'Light leg protection', 1.5, TRUE, 3),
    (33, 'Leather Boots', 6, 'Light foot protection', 0.8, TRUE, 4),

    -- Backpacks
    (40, 'Small Backpack', 3, 'Increases carrying capacity', 1.0, TRUE, 8),
    (41, 'Large Backpack', 3, 'Greatly increases carrying capacity', 2.5, TRUE, 8);

-- Stats des items
INSERT INTO units.item_stat_modifiers (item_id, stat_name, modifier_value) VALUES
    -- Pickaxe bonuses
    (10, 'mining_bonus', 20),

    -- Axe bonuses
    (11, 'lumberjacking_bonus', 20),

    -- Fishing Rod bonuses
    (12, 'fishing_bonus', 20),

    -- Hammer bonuses
    (13, 'blacksmithing_bonus', 20),

    -- Sword bonuses
    (20, 'melee_attack_bonus', 10),
    (20, 'strength_bonus', 2),

    -- Bow bonuses
    (21, 'ranged_attack_bonus', 10),

    -- Shield bonuses
    (22, 'defense_physical', 15),
    (22, 'defense_melee', 20),

    -- Armor bonuses
    (30, 'defense_physical', 3),
    (31, 'defense_physical', 8),
    (32, 'defense_physical', 6),
    (33, 'defense_physical', 4),

    -- Backpack bonuses
    (40, 'inventory_capacity_kg', 20),
    (41, 'inventory_capacity_kg', 50);

-- Update sequence
SELECT setval('units.items_id_seq', GREATEST(50, (SELECT MAX(id) FROM units.items)));
