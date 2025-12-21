-- Remove portrait_variant_id column from units table
ALTER TABLE units.units DROP COLUMN IF EXISTS portrait_variant_id;
