-- Create instance tables for each building category (similar to trees table)

-- Manufacturing Workshops instances
CREATE TABLE IF NOT EXISTS buildings.manufacturing_workshops (
    building_id BIGINT PRIMARY KEY,
    workshop_type_id SMALLINT NOT NULL,
    variant INTEGER NOT NULL DEFAULT 0,
    CONSTRAINT manufacturing_workshops_building_id_fkey
        FOREIGN KEY (building_id) REFERENCES buildings.buildings_base(id) ON DELETE CASCADE,
    CONSTRAINT manufacturing_workshops_workshop_type_id_fkey
        FOREIGN KEY (workshop_type_id) REFERENCES buildings.manufacturing_workshop_types(id)
);

CREATE INDEX IF NOT EXISTS idx_building_manufacturing_workshops ON buildings.manufacturing_workshops(building_id);

-- Agriculture instances
CREATE TABLE IF NOT EXISTS buildings.agriculture (
    building_id BIGINT PRIMARY KEY,
    agriculture_type_id SMALLINT NOT NULL,
    variant INTEGER NOT NULL DEFAULT 0,
    CONSTRAINT agriculture_building_id_fkey
        FOREIGN KEY (building_id) REFERENCES buildings.buildings_base(id) ON DELETE CASCADE,
    CONSTRAINT agriculture_agriculture_type_id_fkey
        FOREIGN KEY (agriculture_type_id) REFERENCES buildings.agriculture_types(id)
);

CREATE INDEX IF NOT EXISTS idx_building_agriculture ON buildings.agriculture(building_id);

-- Animal Breeding instances
CREATE TABLE IF NOT EXISTS buildings.animal_breeding (
    building_id BIGINT PRIMARY KEY,
    animal_type_id SMALLINT NOT NULL,
    variant INTEGER NOT NULL DEFAULT 0,
    CONSTRAINT animal_breeding_building_id_fkey
        FOREIGN KEY (building_id) REFERENCES buildings.buildings_base(id) ON DELETE CASCADE,
    CONSTRAINT animal_breeding_animal_type_id_fkey
        FOREIGN KEY (animal_type_id) REFERENCES buildings.animal_breeding_types(id)
);

CREATE INDEX IF NOT EXISTS idx_building_animal_breeding ON buildings.animal_breeding(building_id);

-- Entertainment instances
CREATE TABLE IF NOT EXISTS buildings.entertainment (
    building_id BIGINT PRIMARY KEY,
    entertainment_type_id SMALLINT NOT NULL,
    variant INTEGER NOT NULL DEFAULT 0,
    CONSTRAINT entertainment_building_id_fkey
        FOREIGN KEY (building_id) REFERENCES buildings.buildings_base(id) ON DELETE CASCADE,
    CONSTRAINT entertainment_entertainment_type_id_fkey
        FOREIGN KEY (entertainment_type_id) REFERENCES buildings.entertainment_types(id)
);

CREATE INDEX IF NOT EXISTS idx_building_entertainment ON buildings.entertainment(building_id);

-- Cult instances
CREATE TABLE IF NOT EXISTS buildings.cult (
    building_id BIGINT PRIMARY KEY,
    cult_type_id SMALLINT NOT NULL,
    variant INTEGER NOT NULL DEFAULT 0,
    CONSTRAINT cult_building_id_fkey
        FOREIGN KEY (building_id) REFERENCES buildings.buildings_base(id) ON DELETE CASCADE,
    CONSTRAINT cult_cult_type_id_fkey
        FOREIGN KEY (cult_type_id) REFERENCES buildings.cult_types(id)
);

CREATE INDEX IF NOT EXISTS idx_building_cult ON buildings.cult(building_id);

-- Commerce instances
CREATE TABLE IF NOT EXISTS buildings.commerce (
    building_id BIGINT PRIMARY KEY,
    commerce_type_id SMALLINT NOT NULL,
    variant INTEGER NOT NULL DEFAULT 0,
    CONSTRAINT commerce_building_id_fkey
        FOREIGN KEY (building_id) REFERENCES buildings.buildings_base(id) ON DELETE CASCADE,
    CONSTRAINT commerce_commerce_type_id_fkey
        FOREIGN KEY (commerce_type_id) REFERENCES buildings.commerce_types(id)
);

CREATE INDEX IF NOT EXISTS idx_building_commerce ON buildings.commerce(building_id);
