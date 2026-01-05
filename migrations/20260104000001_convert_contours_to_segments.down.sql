-- Revert migration: convert segments back to points

-- Rename column from contour_segments back to contour_points
ALTER TABLE organizations.territory_contours
    RENAME COLUMN contour_segments TO contour_points;

-- Rename column from segment_count back to point_count
ALTER TABLE organizations.territory_contours
    RENAME COLUMN segment_count TO point_count;

-- Restore original comments
COMMENT ON COLUMN organizations.territory_contours.contour_points IS 'Flattened array of world coordinates [x1,y1,x2,y2,...]';
COMMENT ON COLUMN organizations.territory_contours.point_count IS 'Number of Vec2 points (length of contour_points / 2)';
