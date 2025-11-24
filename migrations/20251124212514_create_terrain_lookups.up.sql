-- Create terrain lookup tables

CREATE TABLE terrain.biome_types (
    id SMALLINT PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL
);

INSERT INTO terrain.biome_types (id, name) VALUES
    (1, 'Ocean'),
    (2, 'DeepOcean'),
    (3, 'Desert'),
    (4, 'Savanna'),
    (5, 'Grassland'),
    (6, 'TropicalSeasonalForest'),
    (7, 'TropicalRainForest'),
    (8, 'TropicalDeciduousForest'),
    (9, 'TemperateRainForest'),
    (10, 'Wetland'),
    (11, 'Taiga'),
    (12, 'Tundra'),
    (13, 'Lake'),
    (14, 'ColdDesert'),
    (15, 'Ice');
