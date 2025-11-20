-- Add up migration script here
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
    (10, 'Military');

CREATE TABLE buildings.building_specific_types (
    id SMALLINT PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL,
    archived BOOLEAN DEFAULT FALSE
);

INSERT INTO buildings.building_specific_types (id, name) VALUES
    (1, 'Tree'),
    (2, 'ManufacturingWorkshop'),
    (0, 'Unknown');

CREATE TABLE buildings.tree_types (
    id SMALLINT PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL,
    archived BOOLEAN DEFAULT FALSE
);

INSERT INTO buildings.tree_types (id, name) VALUES
    (1, 'Cedar'),
    (2, 'Larch'),
    (3, 'Oak');

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