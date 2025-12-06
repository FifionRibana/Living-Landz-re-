-- Remove cell_path column from road_segments table
ALTER TABLE terrain.road_segments
DROP COLUMN cell_path;
