-- Rollback migration: Remove authentication columns from game.players table
-- Created: 2026-01-10

-- Drop index
DROP INDEX IF EXISTS idx_players_email;

-- Remove columns (in reverse order of creation)
ALTER TABLE game.players DROP COLUMN IF EXISTS last_login_at;
ALTER TABLE game.players DROP COLUMN IF EXISTS account_status;
ALTER TABLE game.players DROP COLUMN IF EXISTS email;
ALTER TABLE game.players DROP COLUMN IF EXISTS password_hash;
