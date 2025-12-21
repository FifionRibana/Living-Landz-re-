-- Migration: Rollback unit slot positions
-- Description: Removes slot_type and slot_index from units table

DROP INDEX IF EXISTS idx_unique_unit_slot;
DROP INDEX IF EXISTS idx_units_cell_slot;

ALTER TABLE units.units
DROP COLUMN IF EXISTS slot_index,
DROP COLUMN IF EXISTS slot_type;
