-- Migration: Add unit slot positions for cell detail view
-- Description: Adds slot_type and slot_index to units table for positioning within cells

-- Add slot position fields to units table
ALTER TABLE units.units
ADD COLUMN slot_type VARCHAR(20),
ADD COLUMN slot_index INT;

-- Index for querying units by cell and slot
CREATE INDEX idx_units_cell_slot ON units.units(
    current_chunk_x, current_chunk_y,
    current_cell_q, current_cell_r,
    slot_type, slot_index
);

-- Constraint: only one unit per slot per cell
CREATE UNIQUE INDEX idx_unique_unit_slot ON units.units(
    current_chunk_x, current_chunk_y,
    current_cell_q, current_cell_r,
    slot_type, slot_index
)
WHERE slot_type IS NOT NULL AND slot_index IS NOT NULL;

COMMENT ON COLUMN units.units.slot_type IS 'Type of slot (interior/exterior) where unit is positioned within the cell';
COMMENT ON COLUMN units.units.slot_index IS 'Index of the slot within the slot type grid (0-based)';
