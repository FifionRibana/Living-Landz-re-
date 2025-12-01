-- Create ocean data table for global ocean SDF and heightmap

CREATE TABLE terrain.ocean_data (
    name VARCHAR(32) NOT NULL PRIMARY KEY,
    width INT NOT NULL,
    height INT NOT NULL,
    max_distance REAL NOT NULL,
    sdf_data BYTEA NOT NULL,
    heightmap_data BYTEA NOT NULL,
    generated_at BIGINT NOT NULL
);

CREATE INDEX idx_ocean_data_generated ON terrain.ocean_data(generated_at);
