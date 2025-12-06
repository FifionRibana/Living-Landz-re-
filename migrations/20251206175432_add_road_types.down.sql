-- Rollback road type tables

-- Remove road_type_id from road_segments
ALTER TABLE terrain.road_segments
DROP COLUMN road_type_id;

-- Drop road types table
DROP TABLE terrain.road_types;

-- Drop road categories table
DROP TABLE terrain.road_categories;
