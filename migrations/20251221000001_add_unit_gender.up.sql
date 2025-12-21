-- Add gender column to units table
ALTER TABLE units.units ADD COLUMN gender VARCHAR(10) DEFAULT 'male' NOT NULL;

-- Add check constraint to ensure valid values
ALTER TABLE units.units ADD CONSTRAINT check_gender CHECK (gender IN ('male', 'female'));
