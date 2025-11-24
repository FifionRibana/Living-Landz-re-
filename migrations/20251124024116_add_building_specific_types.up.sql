-- Add new building specific types
INSERT INTO buildings.building_specific_types (id, name) VALUES
    (3, 'Agriculture'),
    (4, 'AnimalBreeding'),
    (5, 'Entertainment'),
    (6, 'Cult'),
    (7, 'Commerce')
ON CONFLICT (id) DO NOTHING;

-- Update existing building types with correct specific_type_id
UPDATE buildings.building_types SET specific_type_id = 3 WHERE id = 10; -- Farm
UPDATE buildings.building_types SET specific_type_id = 4 WHERE id IN (20, 21, 22, 23); -- Animal Breeding
UPDATE buildings.building_types SET specific_type_id = 5 WHERE id = 30; -- Theater
UPDATE buildings.building_types SET specific_type_id = 6 WHERE id = 40; -- Temple
UPDATE buildings.building_types SET specific_type_id = 7 WHERE id IN (50, 51, 52, 53, 54, 55); -- Commerce

-- Create specific type tables for each building category

-- Manufacturing Workshop Types
CREATE TABLE IF NOT EXISTS buildings.manufacturing_workshop_types (
    id SMALLINT PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL,
    archived BOOLEAN DEFAULT FALSE
);

INSERT INTO buildings.manufacturing_workshop_types (id, name) VALUES
    (1, 'Blacksmith'),
    (2, 'BlastFurnace'),
    (3, 'Bloomery'),
    (4, 'CarpenterShop'),
    (5, 'GlassFactory')
ON CONFLICT (id) DO NOTHING;

-- Agriculture Types
CREATE TABLE IF NOT EXISTS buildings.agriculture_types (
    id SMALLINT PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL,
    archived BOOLEAN DEFAULT FALSE
);

INSERT INTO buildings.agriculture_types (id, name) VALUES
    (10, 'Farm')
ON CONFLICT (id) DO NOTHING;

-- Animal Breeding Types
CREATE TABLE IF NOT EXISTS buildings.animal_breeding_types (
    id SMALLINT PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL,
    archived BOOLEAN DEFAULT FALSE
);

INSERT INTO buildings.animal_breeding_types (id, name) VALUES
    (20, 'Cowshed'),
    (21, 'Piggery'),
    (22, 'Sheepfold'),
    (23, 'Stable')
ON CONFLICT (id) DO NOTHING;

-- Entertainment Types
CREATE TABLE IF NOT EXISTS buildings.entertainment_types (
    id SMALLINT PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL,
    archived BOOLEAN DEFAULT FALSE
);

INSERT INTO buildings.entertainment_types (id, name) VALUES
    (30, 'Theater')
ON CONFLICT (id) DO NOTHING;

-- Cult Types
CREATE TABLE IF NOT EXISTS buildings.cult_types (
    id SMALLINT PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL,
    archived BOOLEAN DEFAULT FALSE
);

INSERT INTO buildings.cult_types (id, name) VALUES
    (40, 'Temple')
ON CONFLICT (id) DO NOTHING;

-- Commerce Types
CREATE TABLE IF NOT EXISTS buildings.commerce_types (
    id SMALLINT PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL,
    archived BOOLEAN DEFAULT FALSE
);

INSERT INTO buildings.commerce_types (id, name) VALUES
    (50, 'Bakehouse'),
    (51, 'Brewery'),
    (52, 'Distillery'),
    (53, 'Slaughterhouse'),
    (54, 'IceHouse'),
    (55, 'Market')
ON CONFLICT (id) DO NOTHING;
