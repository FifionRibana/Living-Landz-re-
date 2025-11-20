-- Add up migration script here
CREATE TABLE buildings.buildings_base (
    id BIGSERIAL PRIMARY KEY,
    building_type_id INT NOT NULL REFERENCES buildings.building_types(id),
    category_id SMALLINT NOT NULL REFERENCES buildings.building_categories(id),
    
    chunk_x INT NOT NULL,
    chunk_y INT NOT NULL,
    cell_q INT NOT NULL,
    cell_r INT NOT NULL,

    quality FLOAT NOT NULL DEFAULT 1.0,
    durability FLOAT NOT NULL DEFAULT 1.0,
    damage FLOAT NOT NULL DEFAULT 0.0,

    created_at BIGINT NOT NULL,

    UNIQUE(cell_q, cell_r),
    UNIQUE(cell_q, cell_r, chunk_x, chunk_y)
);

CREATE INDEX idx_building_type_id ON buildings.buildings_base(building_type_id);
CREATE INDEX idx_buildings_chunk ON buildings.buildings_base(chunk_x, chunk_y);
CREATE INDEX idx_buildings_created ON buildings.buildings_base(created_at);