-- Add up migration script here
ALTER TABLE buildings.buildings_base
ADD COLUMN is_built BOOLEAN NOT NULL DEFAULT true;

-- Créer un index pour optimiser les requêtes qui filtrent par is_built
CREATE INDEX idx_buildings_is_built ON buildings.buildings_base(is_built);

-- Mettre à jour les bâtiments existants (arbres) pour qu'ils soient marqués comme construits
UPDATE buildings.buildings_base
SET is_built = true;
