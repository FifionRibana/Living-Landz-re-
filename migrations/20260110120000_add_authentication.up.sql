-- Migration: Add authentication columns to game.players table
-- Created: 2026-01-10

-- Add password hash column for secure password storage
ALTER TABLE game.players
ADD COLUMN password_hash VARCHAR(255);

-- Add email column for future password recovery (optional)
ALTER TABLE game.players
ADD COLUMN email VARCHAR(255) UNIQUE;

-- Add account status for account management
ALTER TABLE game.players
ADD COLUMN account_status VARCHAR(20) DEFAULT 'active'
CHECK (account_status IN ('active', 'locked', 'suspended'));

-- Add last login tracking
ALTER TABLE game.players
ADD COLUMN last_login_at TIMESTAMPTZ;

-- Create index for email lookups (only for non-null values)
CREATE INDEX idx_players_email ON game.players(email) WHERE email IS NOT NULL;

-- Add comments for documentation
COMMENT ON COLUMN game.players.password_hash IS 'Argon2id password hash in PHC format (includes salt)';
COMMENT ON COLUMN game.players.email IS 'Email address for password recovery (optional)';
COMMENT ON COLUMN game.players.account_status IS 'Account status: active, locked, or suspended';
COMMENT ON COLUMN game.players.last_login_at IS 'Timestamp of last successful login';
