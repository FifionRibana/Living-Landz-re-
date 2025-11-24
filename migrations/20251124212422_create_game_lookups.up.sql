-- Create game lookup tables and main game tables

-- Table pour les langues (lookup)
CREATE TABLE game.languages (
    id SMALLINT PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL,
    code VARCHAR(5) UNIQUE NOT NULL
);

INSERT INTO game.languages (id, name, code) VALUES
    (1, 'French', 'fr'),
    (2, 'English', 'en'),
    (3, 'German', 'de'),
    (4, 'Spanish', 'es'),
    (5, 'Italian', 'it');

-- Table pour les armoiries (lookup)
CREATE TABLE game.coats_of_arms (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    description TEXT,
    image_data BYTEA,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Table principale des joueurs
CREATE TABLE game.players (
    id BIGSERIAL PRIMARY KEY,
    family_name VARCHAR NOT NULL,
    language_id SMALLINT NOT NULL REFERENCES game.languages(id),
    coat_of_arms_id BIGINT REFERENCES game.coats_of_arms(id) ON DELETE SET NULL,
    motto VARCHAR,
    origin_location VARCHAR NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_players_family_name ON game.players(family_name);
CREATE INDEX idx_players_created ON game.players(created_at);
CREATE UNIQUE INDEX idx_players_family_name_unique ON game.players(family_name);

-- Table des personnages associés aux joueurs
CREATE TABLE game.characters (
    id BIGSERIAL PRIMARY KEY,
    player_id BIGINT NOT NULL REFERENCES game.players(id) ON DELETE CASCADE,

    first_name VARCHAR NOT NULL,
    family_name VARCHAR NOT NULL,
    second_name VARCHAR,
    nickname VARCHAR,

    coat_of_arms_id BIGINT REFERENCES game.coats_of_arms(id) ON DELETE SET NULL,
    image_id BIGINT,
    motto VARCHAR,

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(player_id, first_name, family_name)
);

CREATE INDEX idx_characters_player ON game.characters(player_id);
CREATE INDEX idx_characters_name ON game.characters(first_name, family_name);
CREATE INDEX idx_characters_created ON game.characters(created_at);
CREATE INDEX idx_characters_coat_of_arms ON game.characters(coat_of_arms_id);

-- Ajouter des commentaires pour la documentation
COMMENT ON TABLE game.players IS 'Joueurs principaux du jeu';
COMMENT ON COLUMN game.players.family_name IS 'Nom de famille du joueur (nom de la maison/dynastie)';
COMMENT ON COLUMN game.players.language_id IS 'Langue d''origine du joueur';
COMMENT ON COLUMN game.players.coat_of_arms_id IS 'Armoiries principales de la maison';
COMMENT ON COLUMN game.players.motto IS 'Devise de la maison';
COMMENT ON COLUMN game.players.origin_location IS 'Lieu d''origine de la maison';

COMMENT ON TABLE game.characters IS 'Personnages individuels des joueurs';
COMMENT ON COLUMN game.characters.player_id IS 'Joueur propriétaire du personnage';
COMMENT ON COLUMN game.characters.second_name IS 'Deuxième prénom ou nom intermédiaire';
COMMENT ON COLUMN game.characters.family_name IS 'Nom de famille d''usage (peut différer du nom de famille du joueur)';
COMMENT ON COLUMN game.characters.coat_of_arms_id IS 'Armoiries personnelles du caractère';
COMMENT ON COLUMN game.characters.image_id IS 'Référence à une image du personnage';
COMMENT ON COLUMN game.characters.motto IS 'Devise personnelle du personnage';
