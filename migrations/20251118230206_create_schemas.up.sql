CREATE SCHEMA IF NOT EXISTS actions;
CREATE SCHEMA IF NOT EXISTS buildings;
CREATE SCHEMA IF NOT EXISTS terrain;
CREATE SCHEMA IF NOT EXISTS resources;
CREATE SCHEMA IF NOT EXISTS game;

COMMENT ON SCHEMA actions IS 'Système des actions planifiées';
COMMENT ON SCHEMA buildings IS 'Bâtiments et constructions';
COMMENT ON SCHEMA terrain IS 'Terrain, chunks, cells';
COMMENT ON SCHEMA resources IS 'Ressources et économie';
COMMENT ON SCHEMA game IS 'Configuration et constantes du jeu';