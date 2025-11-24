-- Create terrain tables

CREATE TABLE terrain.terrains (
    name VARCHAR(32) NOT NULL,
    chunk_x INT NOT NULL,
    chunk_y INT NOT NULL,
    data BYTEA NOT NULL,
    generated_at BIGINT NOT NULL,
    PRIMARY KEY (name, chunk_x, chunk_y)
);

CREATE INDEX idx_terrains_generated ON terrain.terrains(generated_at);

CREATE TABLE terrain.terrain_biomes (
    name VARCHAR(32) NOT NULL,
    chunk_x INT NOT NULL,
    chunk_y INT NOT NULL,
    biome_id SMALLINT NOT NULL REFERENCES terrain.biome_types(id),
    data BYTEA NOT NULL,
    generated_at BIGINT NOT NULL,
    PRIMARY KEY (name, chunk_x, chunk_y, biome_id)
);

CREATE INDEX idx_terrain_biomes_generated ON terrain.terrain_biomes(generated_at);

CREATE TABLE terrain.cells (
    q INT NOT NULL,
    r INT NOT NULL,

    biome_id SMALLINT NOT NULL REFERENCES terrain.biome_types(id),
    terrain_type VARCHAR,

    building_id BIGINT,

    chunk_x INT NOT NULL,
    chunk_y INT NOT NULL,

    UNIQUE(q, r),
    UNIQUE(chunk_x, chunk_y, q, r),
    PRIMARY KEY (q, r)
);

CREATE INDEX idx_cells_chunk ON terrain.cells(chunk_x, chunk_y);
CREATE INDEX idx_cells_building ON terrain.cells(building_id);
