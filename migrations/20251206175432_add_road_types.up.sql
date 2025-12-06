-- Create road type lookup tables

-- Road categories (DirtPath, PavedRoad, Highway)
CREATE TABLE terrain.road_categories (
    id SMALLINT PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL
);

INSERT INTO terrain.road_categories (id, name) VALUES
    (0, 'DirtPath'),
    (1, 'PavedRoad'),
    (2, 'Highway');

-- Road types (catégorie + variante)
CREATE TABLE terrain.road_types (
    id SERIAL PRIMARY KEY,
    category_id SMALLINT NOT NULL REFERENCES terrain.road_categories(id),
    variant VARCHAR NOT NULL,
    archived BOOLEAN DEFAULT FALSE,
    UNIQUE (category_id, variant)
);

-- Types de routes par défaut
INSERT INTO terrain.road_types (id, category_id, variant) VALUES
    (1, 0, 'basic'),      -- Chemin de terre basique
    (2, 1, 'stone'),      -- Route pavée en pierre
    (3, 2, 'cobblestone'); -- Grande route en pavés

-- Add road_type_id to road_segments table
ALTER TABLE terrain.road_segments
ADD COLUMN road_type_id INT REFERENCES terrain.road_types(id) DEFAULT 1;

-- Update existing segments to use default dirt path
UPDATE terrain.road_segments SET road_type_id = 1 WHERE road_type_id IS NULL;

-- Make road_type_id NOT NULL after setting defaults
ALTER TABLE terrain.road_segments
ALTER COLUMN road_type_id SET NOT NULL;
