-- Rollback: Remove lord support from units

DROP INDEX IF EXISTS units.idx_units_one_lord_per_player;
ALTER TABLE units.units DROP COLUMN IF EXISTS portrait_layers;
ALTER TABLE units.units DROP COLUMN IF EXISTS is_lord;
