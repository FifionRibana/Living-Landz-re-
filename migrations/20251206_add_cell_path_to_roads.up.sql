-- Add cell_path column to road_segments table
-- Stores the complete path of cells that the road passes through
-- Format: BYTEA encoded Vec<(i32, i32)> using bincode

ALTER TABLE terrain.road_segments
ADD COLUMN cell_path BYTEA;
