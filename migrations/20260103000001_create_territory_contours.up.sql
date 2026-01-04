-- Create table to store territory contour points per organization per chunk
CREATE TABLE organizations.territory_contours (
    id BIGSERIAL PRIMARY KEY,
    organization_id BIGINT NOT NULL REFERENCES organizations.organizations(id) ON DELETE CASCADE,
    chunk_x INT NOT NULL,
    chunk_y INT NOT NULL,
    -- Store contour points as array of floats [x1,y1,x2,y2,...]
    -- Using REAL[] for world coordinates
    contour_points REAL[] NOT NULL,
    -- Pre-computed bounding box for quick intersection tests
    bbox_min_x REAL NOT NULL,
    bbox_min_y REAL NOT NULL,
    bbox_max_x REAL NOT NULL,
    bbox_max_y REAL NOT NULL,
    -- Metadata
    point_count INT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    -- Unique constraint: one contour per org per chunk
    UNIQUE (organization_id, chunk_x, chunk_y)
);

-- Indexes for efficient querying
CREATE INDEX idx_territory_contours_org ON organizations.territory_contours(organization_id);
CREATE INDEX idx_territory_contours_chunk ON organizations.territory_contours(chunk_x, chunk_y);

-- Fonction pour mettre à jour updated_at automatiquement
CREATE OR REPLACE FUNCTION organizations.update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to auto-update updated_at
CREATE TRIGGER update_territory_contours_updated_at
    BEFORE UPDATE ON organizations.territory_contours
    FOR EACH ROW
    EXECUTE FUNCTION organizations.update_updated_at_column();

-- Comments
COMMENT ON TABLE organizations.territory_contours IS 'Stores territory contour points per organization per chunk for client rendering';
COMMENT ON COLUMN organizations.territory_contours.contour_points IS 'Flattened array of world coordinates [x1,y1,x2,y2,...]';
COMMENT ON COLUMN organizations.territory_contours.bbox_min_x IS 'Minimum X coordinate of bounding box';
COMMENT ON COLUMN organizations.territory_contours.point_count IS 'Number of Vec2 points (length of contour_points / 2)';
