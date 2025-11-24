-- Create building lookup tables

-- Building categories
CREATE TABLE buildings.building_categories (
    id SMALLINT PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL
);

INSERT INTO buildings.building_categories (id, name) VALUES
    (0, 'Unknown'),
    (1, 'Natural'),
    (2, 'Urbanism'),
    (3, 'Cult'),
    (4, 'Dwellings'),
    (5, 'ManufacturingWorkshops'),
    (6, 'Entertainment'),
    (7, 'Agriculture'),
    (8, 'AnimalBreeding'),
    (9, 'Education'),
    (10, 'Military'),
    (11, 'Commerce');

-- Building specific types
CREATE TABLE buildings.building_specific_types (
    id SMALLINT PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL,
    archived BOOLEAN DEFAULT FALSE
);

INSERT INTO buildings.building_specific_types (id, name) VALUES
    (0, 'Unknown'),
    (1, 'Tree'),
    (2, 'ManufacturingWorkshop'),
    (3, 'Agriculture'),
    (4, 'AnimalBreeding'),
    (5, 'Entertainment'),
    (6, 'Cult'),
    (7, 'Commerce');

-- Tree types
CREATE TABLE buildings.tree_types (
    id SMALLINT PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL,
    archived BOOLEAN DEFAULT FALSE
);

INSERT INTO buildings.tree_types (id, name) VALUES
    (1, 'Cedar'),
    (2, 'Larch'),
    (3, 'Oak');

-- Manufacturing Workshop types
CREATE TABLE buildings.manufacturing_workshop_types (
    id SMALLINT PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL,
    archived BOOLEAN DEFAULT FALSE
);

INSERT INTO buildings.manufacturing_workshop_types (id, name) VALUES
    (1, 'Blacksmith'),
    (2, 'BlastFurnace'),
    (3, 'Bloomery'),
    (4, 'CarpenterShop'),
    (5, 'GlassFactory');

-- Agriculture types
CREATE TABLE buildings.agriculture_types (
    id SMALLINT PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL,
    archived BOOLEAN DEFAULT FALSE
);

INSERT INTO buildings.agriculture_types (id, name) VALUES
    (10, 'Farm');

-- Animal Breeding types
CREATE TABLE buildings.animal_breeding_types (
    id SMALLINT PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL,
    archived BOOLEAN DEFAULT FALSE
);

INSERT INTO buildings.animal_breeding_types (id, name) VALUES
    (20, 'Cowshed'),
    (21, 'Piggery'),
    (22, 'Sheepfold'),
    (23, 'Stable');

-- Entertainment types
CREATE TABLE buildings.entertainment_types (
    id SMALLINT PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL,
    archived BOOLEAN DEFAULT FALSE
);

INSERT INTO buildings.entertainment_types (id, name) VALUES
    (30, 'Theater');

-- Cult types
CREATE TABLE buildings.cult_types (
    id SMALLINT PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL,
    archived BOOLEAN DEFAULT FALSE
);

INSERT INTO buildings.cult_types (id, name) VALUES
    (40, 'Temple');

-- Commerce types
CREATE TABLE buildings.commerce_types (
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
    (55, 'Market');

-- Building types (main lookup table)
CREATE TABLE buildings.building_types (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    category_id SMALLINT NOT NULL REFERENCES buildings.building_categories(id),
    specific_type_id SMALLINT NOT NULL REFERENCES buildings.building_specific_types(id),
    description TEXT,
    archived BOOLEAN DEFAULT FALSE,
    UNIQUE(name, category_id, specific_type_id)
);

CREATE INDEX idx_building_types_category ON buildings.building_types(category_id);
CREATE INDEX idx_building_types_specific ON buildings.building_types(specific_type_id);

-- Insert all building types
INSERT INTO buildings.building_types (id, name, category_id, specific_type_id, description) VALUES
    -- ManufacturingWorkshops (category 5, specific_type 2)
    (1, 'Blacksmith', 5, 2, 'Workshop for metalworking and tool crafting'),
    (2, 'Blast Furnace', 5, 2, 'Advanced furnace for smelting iron ore'),
    (3, 'Bloomery', 5, 2, 'Primitive iron smelting facility'),
    (4, 'Carpenter Shop', 5, 2, 'Workshop for woodworking and furniture'),
    (5, 'Glass Factory', 5, 2, 'Facility for glass production'),

    -- Agriculture (category 7, specific_type 3)
    (10, 'Farm', 7, 3, 'Agricultural land for crop production'),

    -- AnimalBreeding (category 8, specific_type 4)
    (20, 'Cowshed', 8, 4, 'Shelter for cattle'),
    (21, 'Piggery', 8, 4, 'Enclosure for raising pigs'),
    (22, 'Sheepfold', 8, 4, 'Enclosure for sheep'),
    (23, 'Stable', 8, 4, 'Building for housing horses'),

    -- Entertainment (category 6, specific_type 5)
    (30, 'Theater', 6, 5, 'Building for dramatic performances'),

    -- Cult (category 3, specific_type 6)
    (40, 'Temple', 3, 6, 'Religious building for worship'),

    -- Commerce (category 11, specific_type 7)
    (50, 'Bakehouse', 11, 7, 'Building for baking bread and pastries'),
    (51, 'Brewery', 11, 7, 'Facility for brewing beer and ale'),
    (52, 'Distillery', 11, 7, 'Facility for distilling spirits'),
    (53, 'Slaughterhouse', 11, 7, 'Facility for butchering livestock'),
    (54, 'Ice House', 11, 7, 'Cold storage building'),
    (55, 'Market', 11, 7, 'Trading building for goods exchange')
ON CONFLICT (id) DO NOTHING;

-- Update the sequence to avoid conflicts with manually inserted IDs
SELECT setval('buildings.building_types_id_seq', GREATEST(55, (SELECT MAX(id) FROM buildings.building_types)));
