-- Migration: Create organizations schema and lookup tables
-- Description: Defines the base structure for organizations (villages, cities, kingdoms, guilds, etc.)

-- Create schema
CREATE SCHEMA IF NOT EXISTS organizations;
COMMENT ON SCHEMA organizations IS 'Système des organisations (territoires, guildes, ordres religieux, etc.)';

-- ============================================================================
-- LOOKUP TABLES
-- ============================================================================

-- Organization types lookup
CREATE TABLE organizations.organization_types (
    id SMALLINT PRIMARY KEY,
    name VARCHAR(50) NOT NULL UNIQUE,
    category VARCHAR(20) NOT NULL, -- 'territorial', 'religious', 'commercial', 'social'
    requires_territory BOOLEAN NOT NULL DEFAULT true,
    can_have_vassals BOOLEAN NOT NULL DEFAULT false,
    can_have_parent BOOLEAN NOT NULL DEFAULT false,
    min_population INT,
    min_area_km2 DECIMAL(10, 2),
    description TEXT
);

COMMENT ON TABLE organizations.organization_types IS 'Types d''organisations disponibles';
COMMENT ON COLUMN organizations.organization_types.category IS 'Catégorie: territorial, religious, commercial, social';
COMMENT ON COLUMN organizations.organization_types.requires_territory IS 'Nécessite un territoire pour exister';
COMMENT ON COLUMN organizations.organization_types.can_have_vassals IS 'Peut avoir des vassaux/organisations subordonnées';
COMMENT ON COLUMN organizations.organization_types.can_have_parent IS 'Peut avoir une organisation parente (vassalité)';

-- Insert organization types
INSERT INTO organizations.organization_types (id, name, category, requires_territory, can_have_vassals, can_have_parent, min_population, min_area_km2, description) VALUES
    -- Territorial organizations (1-20)
    (1, 'Hamlet', 'territorial', true, false, true, 1, 0.0, 'Small rural settlement'),
    (2, 'Village', 'territorial', true, false, true, 50, 2.0, 'Rural settlement with basic facilities'),
    (3, 'Town', 'territorial', true, true, true, 200, 5.0, 'Larger settlement with markets and workshops'),
    (4, 'City', 'territorial', true, true, true, 1000, 10.0, 'Major urban center with diverse economy'),
    (5, 'Barony', 'territorial', true, true, true, 500, 50.0, 'Noble territory ruled by a baron'),
    (6, 'County', 'territorial', true, true, true, 2000, 200.0, 'Territory ruled by a count'),
    (7, 'Duchy', 'territorial', true, true, true, 10000, 1000.0, 'Large territory ruled by a duke'),
    (8, 'Kingdom', 'territorial', true, true, false, 50000, 5000.0, 'Sovereign state ruled by a king'),
    (9, 'Empire', 'territorial', true, true, false, 200000, 20000.0, 'Multi-kingdom state ruled by an emperor'),

    -- Religious organizations (20-39)
    (20, 'Chapel', 'religious', false, false, true, 10, NULL, 'Small place of worship'),
    (21, 'Church', 'religious', false, false, true, 50, NULL, 'Parish church serving a community'),
    (22, 'Abbey', 'religious', false, true, true, 20, NULL, 'Monastery with religious community'),
    (23, 'Diocese', 'religious', false, true, true, 1000, NULL, 'Ecclesiastical district under a bishop'),
    (24, 'Archdiocese', 'religious', false, true, false, 10000, NULL, 'Metropolitan diocese under an archbishop'),
    (25, 'Temple', 'religious', false, false, true, 30, NULL, 'Sacred building for worship'),
    (26, 'Monastery', 'religious', false, true, true, 15, NULL, 'Religious community living under vows'),

    -- Commercial organizations (40-59)
    (40, 'Shop', 'commercial', false, false, true, 1, NULL, 'Small retail establishment'),
    (41, 'Workshop', 'commercial', false, false, true, 3, NULL, 'Artisan production facility'),
    (42, 'Trading Post', 'commercial', false, false, true, 5, NULL, 'Regional trading hub'),
    (43, 'Market', 'commercial', false, false, true, 10, NULL, 'Organized marketplace'),
    (44, 'Merchant Guild', 'commercial', false, true, true, 20, NULL, 'Association of merchants'),
    (45, 'Trading Company', 'commercial', false, true, false, 50, NULL, 'Large-scale commercial enterprise'),
    (46, 'Bank', 'commercial', false, false, true, 10, NULL, 'Financial institution'),

    -- Social/Military organizations (60-79)
    (60, 'Militia', 'social', false, false, true, 10, NULL, 'Local defense force'),
    (61, 'Mercenary Band', 'social', false, false, true, 15, NULL, 'Professional soldiers for hire'),
    (62, 'Knight Order', 'social', false, true, true, 30, NULL, 'Chivalric military order'),
    (63, 'Crafters Guild', 'social', false, true, true, 10, NULL, 'Association of craftspeople'),
    (64, 'Scholars Guild', 'social', false, true, true, 8, NULL, 'Association of learned individuals'),
    (65, 'Thieves Guild', 'social', false, true, true, 15, NULL, 'Organized criminal network'),
    (66, 'Army', 'social', false, false, true, 100, NULL, 'Organized military force');

-- ============================================================================
-- Role types lookup
CREATE TABLE organizations.role_types (
    id SMALLINT PRIMARY KEY,
    name VARCHAR(50) NOT NULL UNIQUE,
    category VARCHAR(20) NOT NULL,
    authority_level SMALLINT NOT NULL, -- 1=highest (Emperor/Pope), 100=lowest
    description TEXT
);

COMMENT ON TABLE organizations.role_types IS 'Rôles/postes disponibles dans les organisations';
COMMENT ON COLUMN organizations.role_types.authority_level IS 'Niveau d''autorité: 1=le plus élevé, 100=le plus bas';

-- Insert role types
INSERT INTO organizations.role_types (id, name, category, authority_level, description) VALUES
    -- Territorial leaders (1-19)
    (1, 'Emperor', 'territorial', 1, 'Supreme ruler of an empire'),
    (2, 'King', 'territorial', 5, 'Sovereign ruler of a kingdom'),
    (3, 'Duke', 'territorial', 10, 'Ruler of a duchy'),
    (4, 'Count', 'territorial', 15, 'Ruler of a county'),
    (5, 'Baron', 'territorial', 20, 'Ruler of a barony'),
    (6, 'Mayor', 'territorial', 25, 'Leader of a town or city'),
    (7, 'Village Elder', 'territorial', 30, 'Leader of a village'),
    (8, 'Headman', 'territorial', 35, 'Leader of a hamlet'),

    -- Territorial officers (20-39)
    (20, 'Chancellor', 'territorial', 12, 'Chief administrator and advisor'),
    (21, 'Marshal', 'territorial', 13, 'Military commander'),
    (22, 'Steward', 'territorial', 14, 'Manager of finances and resources'),
    (23, 'Treasurer', 'territorial', 16, 'Keeper of the treasury'),
    (24, 'Deputy Mayor', 'territorial', 26, 'Second-in-command to the mayor'),
    (25, 'Sheriff', 'territorial', 28, 'Law enforcement officer'),
    (26, 'Tax Collector', 'territorial', 40, 'Collects taxes for the realm'),

    -- Religious leaders (40-59)
    (40, 'Pope', 'religious', 1, 'Supreme pontiff'),
    (41, 'Archbishop', 'religious', 8, 'Metropolitan bishop'),
    (42, 'Bishop', 'religious', 18, 'Overseer of a diocese'),
    (43, 'Abbot', 'religious', 22, 'Head of an abbey'),
    (44, 'Priest', 'religious', 32, 'Leader of a church or chapel'),
    (45, 'Cardinal', 'religious', 10, 'High-ranking church official'),
    (46, 'Prior', 'religious', 24, 'Deputy to an abbot'),
    (47, 'Chaplain', 'religious', 38, 'Religious advisor'),

    -- Commercial leaders (60-79)
    (60, 'Guild Master', 'commercial', 15, 'Head of a guild'),
    (61, 'Trade Director', 'commercial', 17, 'Director of trading company'),
    (62, 'Shopkeeper', 'commercial', 45, 'Owner of a shop'),
    (63, 'Workshop Master', 'commercial', 35, 'Owner of a workshop'),
    (64, 'Trade Treasurer', 'commercial', 25, 'Financial officer of trade organization'),
    (65, 'Market Master', 'commercial', 30, 'Overseer of a marketplace'),
    (66, 'Banker', 'commercial', 20, 'Head of a bank'),

    -- Military/Social leaders (80-99)
    (80, 'Grand Master', 'military', 11, 'Supreme leader of a knight order'),
    (81, 'Commander', 'military', 19, 'Military commander'),
    (82, 'Captain', 'military', 33, 'Leader of a military unit'),
    (83, 'Lieutenant', 'military', 42, 'Second-in-command to a captain'),
    (84, 'Sergeant', 'military', 50, 'Non-commissioned officer'),
    (85, 'Master Craftsman', 'social', 27, 'Leader of crafters guild'),
    (86, 'Scholar Master', 'social', 29, 'Leader of scholars guild'),
    (87, 'Guildmaster Thief', 'social', 23, 'Leader of thieves guild');

-- ============================================================================
-- Organization type - Role compatibility
-- Defines which roles are valid for which organization types
CREATE TABLE organizations.organization_role_compatibility (
    organization_type_id SMALLINT NOT NULL REFERENCES organizations.organization_types(id),
    role_type_id SMALLINT NOT NULL REFERENCES organizations.role_types(id),
    is_leader_role BOOLEAN NOT NULL DEFAULT false, -- Can this role be the main leader?
    PRIMARY KEY (organization_type_id, role_type_id)
);

COMMENT ON TABLE organizations.organization_role_compatibility IS 'Définit quels rôles sont compatibles avec quels types d''organisation';

-- Insert compatibility mappings
-- Territorial organizations
INSERT INTO organizations.organization_role_compatibility (organization_type_id, role_type_id, is_leader_role) VALUES
    -- Hamlet
    (1, 8, true), (1, 26, false),
    -- Village
    (2, 7, true), (2, 26, false),
    -- Town
    (3, 6, true), (3, 24, false), (3, 25, false), (3, 23, false),
    -- City
    (4, 6, true), (4, 24, false), (4, 25, false), (4, 23, false), (4, 26, false),
    -- Barony
    (5, 5, true), (5, 22, false), (5, 21, false), (5, 23, false),
    -- County
    (6, 4, true), (6, 22, false), (6, 21, false), (6, 20, false), (6, 23, false),
    -- Duchy
    (7, 3, true), (7, 20, false), (7, 21, false), (7, 22, false), (7, 23, false),
    -- Kingdom
    (8, 2, true), (8, 20, false), (8, 21, false), (8, 22, false), (8, 23, false),
    -- Empire
    (9, 1, true), (9, 20, false), (9, 21, false), (9, 22, false), (9, 23, false),

    -- Religious organizations
    (20, 44, true), (20, 47, false), -- Chapel
    (21, 44, true), (21, 47, false), -- Church
    (22, 43, true), (22, 46, false), -- Abbey
    (23, 42, true), (23, 47, false), (23, 44, false), -- Diocese
    (24, 41, true), (24, 45, false), (24, 42, false), -- Archdiocese
    (25, 44, true), (25, 47, false), -- Temple
    (26, 43, true), (26, 46, false), -- Monastery

    -- Commercial organizations
    (40, 62, true), -- Shop
    (41, 63, true), -- Workshop
    (42, 62, true), (42, 64, false), -- Trading Post
    (43, 65, true), (43, 64, false), -- Market
    (44, 60, true), (44, 64, false), -- Merchant Guild
    (45, 61, true), (45, 64, false), -- Trading Company
    (46, 66, true), (46, 64, false), -- Bank

    -- Social/Military organizations
    (60, 82, true), (60, 83, false), -- Militia
    (61, 81, true), (61, 82, false), -- Mercenary Band
    (62, 80, true), (62, 81, false), (62, 82, false), -- Knight Order
    (63, 85, true), -- Crafters Guild
    (64, 86, true), -- Scholars Guild
    (65, 87, true), -- Thieves Guild
    (66, 81, true), (66, 82, false), (66, 83, false); -- Army
