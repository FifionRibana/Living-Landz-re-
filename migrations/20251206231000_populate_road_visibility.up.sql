-- Populate road_chunk_visibility for existing roads
-- This migration recalculates which chunks each road segment traverses

-- Fonction helper pour convertir une cellule en chunk
-- Doit être synchronisée avec la fonction Rust cell_to_chunk
CREATE OR REPLACE FUNCTION terrain.cell_to_chunk(cell_q INT, cell_r INT)
RETURNS TABLE(chunk_x INT, chunk_y INT) AS $$
DECLARE
    hex_size FLOAT := 16.0;
    hex_ratio_x FLOAT := 1.0;
    hex_ratio_y FLOAT := 0.866025404;
    chunk_size_x FLOAT := 600.0;
    chunk_size_y FLOAT := 503.0;
    world_x FLOAT;
    world_y FLOAT;
BEGIN
    -- Conversion hex flat-top layout
    world_x := hex_size * hex_ratio_x * (1.5 * cell_q);
    world_y := hex_size * hex_ratio_y * (SQRT(3.0) * (cell_r + cell_q / 2.0));

    -- Calculer le chunk
    chunk_x := FLOOR(world_x / chunk_size_x)::INT;
    chunk_y := FLOOR(world_y / chunk_size_y)::INT;

    RETURN NEXT;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- Vider la table de visibilité existante
TRUNCATE TABLE terrain.road_chunk_visibility;

-- Peupler avec les données calculées
-- Pour chaque segment, on récupère tous les chunks traversés via le cell_path
INSERT INTO terrain.road_chunk_visibility (segment_id, chunk_x, chunk_y, is_endpoint)
SELECT DISTINCT
    rs.id AS segment_id,
    c.chunk_x,
    c.chunk_y,
    (c.chunk_x = start_chunk.chunk_x AND c.chunk_y = start_chunk.chunk_y) OR
    (c.chunk_x = end_chunk.chunk_x AND c.chunk_y = end_chunk.chunk_y) AS is_endpoint
FROM terrain.road_segments rs
CROSS JOIN LATERAL (
    -- Extraire les cellules du cell_path (stocké en bincode)
    -- Pour l'instant, on utilise juste le chunk du start/end
    -- TODO: parser le cell_path bincode pour une précision complète
    SELECT * FROM terrain.cell_to_chunk(rs.start_q, rs.start_r)
    UNION
    SELECT * FROM terrain.cell_to_chunk(rs.end_q, rs.end_r)
) c
CROSS JOIN LATERAL terrain.cell_to_chunk(rs.start_q, rs.start_r) start_chunk
CROSS JOIN LATERAL terrain.cell_to_chunk(rs.end_q, rs.end_r) end_chunk;

-- Nettoyer la fonction temporaire si nécessaire (optionnel)
-- DROP FUNCTION IF EXISTS terrain.cell_to_chunk(INT, INT);
