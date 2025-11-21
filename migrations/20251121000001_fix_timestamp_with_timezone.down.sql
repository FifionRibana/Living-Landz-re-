-- Revenir aux colonnes TIMESTAMP (sans timezone) pour game.coats_of_arms
ALTER TABLE game.coats_of_arms
    ALTER COLUMN created_at TYPE TIMESTAMP USING created_at AT TIME ZONE 'UTC';

-- Revenir aux colonnes TIMESTAMP (sans timezone) pour game.players
ALTER TABLE game.players
    ALTER COLUMN created_at TYPE TIMESTAMP USING created_at AT TIME ZONE 'UTC',
    ALTER COLUMN updated_at TYPE TIMESTAMP USING updated_at AT TIME ZONE 'UTC';

-- Revenir aux colonnes TIMESTAMP (sans timezone) pour game.characters
ALTER TABLE game.characters
    ALTER COLUMN created_at TYPE TIMESTAMP USING created_at AT TIME ZONE 'UTC',
    ALTER COLUMN updated_at TYPE TIMESTAMP USING updated_at AT TIME ZONE 'UTC';
