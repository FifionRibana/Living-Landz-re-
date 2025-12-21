-- Remove gender column from units table
ALTER TABLE units.units DROP CONSTRAINT IF EXISTS check_gender;
ALTER TABLE units.units DROP COLUMN IF EXISTS gender;
