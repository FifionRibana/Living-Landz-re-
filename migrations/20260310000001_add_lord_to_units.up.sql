-- Migration: Add lord support to units
-- The lord is the player's avatar on the map. One lord per player.

-- Flag to mark the player's main unit (lord/lady)
ALTER TABLE units.units
ADD COLUMN is_lord BOOLEAN NOT NULL DEFAULT false;

-- Portrait layer selections (JSON-encoded, e.g. "0,3,1,2" for bust,face,clothes,hair indices)
ALTER TABLE units.units
ADD COLUMN portrait_layers TEXT;

-- Enforce: at most one lord per player
CREATE UNIQUE INDEX idx_units_one_lord_per_player
ON units.units(player_id)
WHERE is_lord = true AND player_id IS NOT NULL;

COMMENT ON COLUMN units.units.is_lord IS 'True if this unit is the player''s Lord/Lady (main avatar on the map)';
COMMENT ON COLUMN units.units.portrait_layers IS 'Portrait layer indices as comma-separated values: bust,face,clothes,hair';
