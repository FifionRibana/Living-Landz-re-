-- Drop trigger and function
DROP TRIGGER IF EXISTS trigger_update_zone_cell_count ON terrain.voronoi_zone_cells;
DROP FUNCTION IF EXISTS update_zone_cell_count();

-- Drop tables (cascade will handle foreign keys)
DROP TABLE IF EXISTS terrain.voronoi_zone_cells;
DROP TABLE IF EXISTS terrain.voronoi_zones;
