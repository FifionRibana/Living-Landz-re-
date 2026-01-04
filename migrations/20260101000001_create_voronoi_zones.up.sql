-- Table des zones Voronoi
CREATE TABLE terrain.voronoi_zones (
    id BIGSERIAL PRIMARY KEY,
    seed_cell_q INT NOT NULL,
    seed_cell_r INT NOT NULL,
    biome_type INT NOT NULL,
    cell_count INT DEFAULT 0,
    area_m2 REAL DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE (seed_cell_q, seed_cell_r)
);

CREATE INDEX idx_voronoi_zones_seed ON terrain.voronoi_zones(seed_cell_q, seed_cell_r);
CREATE INDEX idx_voronoi_zones_biome ON terrain.voronoi_zones(biome_type);

-- Mapping cellule → zone
CREATE TABLE terrain.voronoi_zone_cells (
    zone_id BIGINT NOT NULL REFERENCES terrain.voronoi_zones(id) ON DELETE CASCADE,
    cell_q INT NOT NULL,
    cell_r INT NOT NULL,
    PRIMARY KEY (cell_q, cell_r)
);

CREATE INDEX idx_voronoi_zone_cells_zone ON terrain.voronoi_zone_cells(zone_id);

-- Trigger pour compter les cellules
CREATE OR REPLACE FUNCTION update_zone_cell_count()
RETURNS TRIGGER AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE terrain.voronoi_zones
        SET cell_count = cell_count + 1,
            area_m2 = (cell_count + 1) * 50.0
        WHERE id = NEW.zone_id;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE terrain.voronoi_zones
        SET cell_count = cell_count - 1,
            area_m2 = (cell_count - 1) * 50.0
        WHERE id = OLD.zone_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_zone_cell_count
AFTER INSERT OR DELETE ON terrain.voronoi_zone_cells
FOR EACH ROW EXECUTE FUNCTION update_zone_cell_count();
