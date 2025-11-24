-- Create resource lookup tables

CREATE TABLE resources.resource_categories (
    id SMALLINT PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL
);

INSERT INTO resources.resource_categories (id, name) VALUES
    (0, 'Unknown'),
    (1, 'Wood'),
    (2, 'Metal'),
    (3, 'CrudeMaterial'),
    (4, 'Food'),
    (5, 'Furniture'),
    (6, 'Weaponry'),
    (7, 'Jewelry'),
    (8, 'Meat'),
    (9, 'Fruits'),
    (10, 'Vegetables');

CREATE TABLE resources.resource_specific_types (
    id SMALLINT PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL,
    category_id SMALLINT NOT NULL REFERENCES resources.resource_categories(id),
    archived BOOLEAN DEFAULT FALSE
);

INSERT INTO resources.resource_specific_types (id, name, category_id) VALUES
    (0, 'Unknown', 0),
    (1, 'Wood', 1),
    (2, 'Ore', 2),
    (3, 'Metal', 3),
    (4, 'Mineral', 4);

CREATE TABLE resources.resource_types (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL UNIQUE,
    category_id SMALLINT NOT NULL REFERENCES resources.resource_categories(id),
    specific_type_id SMALLINT NOT NULL REFERENCES resources.resource_specific_types(id),
    description TEXT,
    archived BOOLEAN DEFAULT FALSE,
    UNIQUE(name, category_id, specific_type_id)
);

CREATE INDEX idx_resource_types_category ON resources.resource_types(category_id);
CREATE INDEX idx_resource_types_specific ON resources.resource_types(specific_type_id);
