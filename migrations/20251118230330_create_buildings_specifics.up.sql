-- Add up migration script here
CREATE TABLE buildings.trees (
    building_id BIGINT PRIMARY KEY REFERENCES buildings.buildings_base(id) ON DELETE CASCADE,
    tree_type_id SMALLINT NOT NULL REFERENCES buildings.tree_types(id),
    density INT NOT NULL,
    age INT NOT NULL,
    variant INT NOT NULL
);

CREATE INDEX idx_building_trees ON buildings.trees(building_id);

CREATE TABLE buildings.manufacturing_workshops (
    building_id BIGINT PRIMARY KEY REFERENCES buildings.buildings_base(id) ON DELETE CASCADE
);