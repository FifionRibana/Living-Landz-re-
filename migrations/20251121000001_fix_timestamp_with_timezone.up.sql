-- Convertir les colonnes TIMESTAMP en TIMESTAMPTZ pour game.coats_of_arms
ALTER TABLE game.coats_of_arms
    ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at AT TIME ZONE 'UTC';

-- Convertir les colonnes TIMESTAMP en TIMESTAMPTZ pour game.players
ALTER TABLE game.players
    ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at AT TIME ZONE 'UTC',
    ALTER COLUMN updated_at TYPE TIMESTAMPTZ USING updated_at AT TIME ZONE 'UTC';

-- Convertir les colonnes TIMESTAMP en TIMESTAMPTZ pour game.characters
ALTER TABLE game.characters
    ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at AT TIME ZONE 'UTC',
    ALTER COLUMN updated_at TYPE TIMESTAMPTZ USING updated_at AT TIME ZONE 'UTC';
