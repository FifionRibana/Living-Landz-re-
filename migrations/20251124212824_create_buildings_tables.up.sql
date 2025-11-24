-- Create buildings tables

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

    is_built BOOLEAN NOT NULL DEFAULT true,

    created_at BIGINT NOT NULL,

    UNIQUE(cell_q, cell_r),
    UNIQUE(cell_q, cell_r, chunk_x, chunk_y)
);

CREATE INDEX idx_building_type_id ON buildings.buildings_base(building_type_id);
CREATE INDEX idx_buildings_chunk ON buildings.buildings_base(chunk_x, chunk_y);
CREATE INDEX idx_buildings_created ON buildings.buildings_base(created_at);
CREATE INDEX idx_buildings_is_built ON buildings.buildings_base(is_built);

-- Trees specific table
CREATE TABLE buildings.trees (
    building_id BIGINT PRIMARY KEY REFERENCES buildings.buildings_base(id) ON DELETE CASCADE,
    tree_type_id SMALLINT NOT NULL REFERENCES buildings.tree_types(id),
    density INT NOT NULL,
    age INT NOT NULL,
    variant INT NOT NULL
);

CREATE INDEX idx_building_trees ON buildings.trees(building_id);

-- Manufacturing Workshops specific table
CREATE TABLE buildings.manufacturing_workshops (
    building_id BIGINT PRIMARY KEY,
    workshop_type_id SMALLINT NOT NULL,
    variant INTEGER NOT NULL DEFAULT 0,
    CONSTRAINT manufacturing_workshops_building_id_fkey
        FOREIGN KEY (building_id) REFERENCES buildings.buildings_base(id) ON DELETE CASCADE,
    CONSTRAINT manufacturing_workshops_workshop_type_id_fkey
        FOREIGN KEY (workshop_type_id) REFERENCES buildings.manufacturing_workshop_types(id)
);

CREATE INDEX idx_building_manufacturing_workshops ON buildings.manufacturing_workshops(building_id);

-- Agriculture specific table
CREATE TABLE buildings.agriculture (
    building_id BIGINT PRIMARY KEY,
    agriculture_type_id SMALLINT NOT NULL,
    variant INTEGER NOT NULL DEFAULT 0,
    CONSTRAINT agriculture_building_id_fkey
        FOREIGN KEY (building_id) REFERENCES buildings.buildings_base(id) ON DELETE CASCADE,
    CONSTRAINT agriculture_agriculture_type_id_fkey
        FOREIGN KEY (agriculture_type_id) REFERENCES buildings.agriculture_types(id)
);

CREATE INDEX idx_building_agriculture ON buildings.agriculture(building_id);

-- Animal Breeding specific table
CREATE TABLE buildings.animal_breeding (
    building_id BIGINT PRIMARY KEY,
    animal_type_id SMALLINT NOT NULL,
    variant INTEGER NOT NULL DEFAULT 0,
    CONSTRAINT animal_breeding_building_id_fkey
        FOREIGN KEY (building_id) REFERENCES buildings.buildings_base(id) ON DELETE CASCADE,
    CONSTRAINT animal_breeding_animal_type_id_fkey
        FOREIGN KEY (animal_type_id) REFERENCES buildings.animal_breeding_types(id)
);

CREATE INDEX idx_building_animal_breeding ON buildings.animal_breeding(building_id);

-- Entertainment specific table
CREATE TABLE buildings.entertainment (
    building_id BIGINT PRIMARY KEY,
    entertainment_type_id SMALLINT NOT NULL,
    variant INTEGER NOT NULL DEFAULT 0,
    CONSTRAINT entertainment_building_id_fkey
        FOREIGN KEY (building_id) REFERENCES buildings.buildings_base(id) ON DELETE CASCADE,
    CONSTRAINT entertainment_entertainment_type_id_fkey
        FOREIGN KEY (entertainment_type_id) REFERENCES buildings.entertainment_types(id)
);

CREATE INDEX idx_building_entertainment ON buildings.entertainment(building_id);

-- Cult specific table
CREATE TABLE buildings.cult (
    building_id BIGINT PRIMARY KEY,
    cult_type_id SMALLINT NOT NULL,
    variant INTEGER NOT NULL DEFAULT 0,
    CONSTRAINT cult_building_id_fkey
        FOREIGN KEY (building_id) REFERENCES buildings.buildings_base(id) ON DELETE CASCADE,
    CONSTRAINT cult_cult_type_id_fkey
        FOREIGN KEY (cult_type_id) REFERENCES buildings.cult_types(id)
);

CREATE INDEX idx_building_cult ON buildings.cult(building_id);

-- Commerce specific table
CREATE TABLE buildings.commerce (
    building_id BIGINT PRIMARY KEY,
    commerce_type_id SMALLINT NOT NULL,
    variant INTEGER NOT NULL DEFAULT 0,
    CONSTRAINT commerce_building_id_fkey
        FOREIGN KEY (building_id) REFERENCES buildings.buildings_base(id) ON DELETE CASCADE,
    CONSTRAINT commerce_commerce_type_id_fkey
        FOREIGN KEY (commerce_type_id) REFERENCES buildings.commerce_types(id)
);

CREATE INDEX idx_building_commerce ON buildings.commerce(building_id);

-- Add foreign key constraint from terrain.cells to buildings.buildings_base
ALTER TABLE terrain.cells
ADD CONSTRAINT cells_building_id_fkey
    FOREIGN KEY (building_id) REFERENCES buildings.buildings_base(id) ON DELETE SET NULL;
