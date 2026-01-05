-- Migrate territory_contours table from storing points to storing segments
-- Format changes from [x1,y1,x2,y2,...] to [start.x,start.y,end.x,end.y,normal.x,normal.y,...]

-- Rename column from contour_points to contour_segments
ALTER TABLE organizations.territory_contours
    RENAME COLUMN contour_points TO contour_segments;

-- Rename column from point_count to segment_count
ALTER TABLE organizations.territory_contours
    RENAME COLUMN point_count TO segment_count;

-- Update comments
COMMENT ON COLUMN organizations.territory_contours.contour_segments IS 'Flattened array of segment data [start.x,start.y,end.x,end.y,normal.x,normal.y,...] where each segment has 6 floats';
COMMENT ON COLUMN organizations.territory_contours.segment_count IS 'Number of ContourSegment (length of contour_segments / 6)';

-- Note: Existing data will need to be regenerated using --regen-territory command
-- The old format (points) is incompatible with the new format (segments with normals)
-- Option 1: Run `cargo run --bin server -- --regen-territory` to regenerate all contours
-- Option 2: Delete existing data: DELETE FROM organizations.territory_contours;
