-- Create road-chunk visibility table for efficient chunk-based loading
-- This table maps which road segments are visible in which chunks

-- Table de relation : quel segment de route est visible dans quel chunk
CREATE TABLE terrain.road_chunk_visibility (
    segment_id BIGINT NOT NULL,
    chunk_x INT NOT NULL,
    chunk_y INT NOT NULL,

    -- Métadonnées pour optimisation
    -- Indique si le segment commence ou se termine dans ce chunk
    -- vs simplement le traverse
    is_endpoint BOOLEAN NOT NULL DEFAULT false,

    PRIMARY KEY (segment_id, chunk_x, chunk_y),
    FOREIGN KEY (segment_id) REFERENCES terrain.road_segments(id) ON DELETE CASCADE
);

-- Index pour requêtes par chunk (le cas d'usage principal)
CREATE INDEX idx_road_chunk_visibility_chunk ON terrain.road_chunk_visibility(chunk_x, chunk_y);

-- Index pour retrouver tous les chunks d'un segment
CREATE INDEX idx_road_chunk_visibility_segment ON terrain.road_chunk_visibility(segment_id);

-- Migrer les données existantes
-- Pour chaque segment existant, créer une entrée de visibilité dans son chunk principal
INSERT INTO terrain.road_chunk_visibility (segment_id, chunk_x, chunk_y, is_endpoint)
SELECT id, chunk_x, chunk_y, true
FROM terrain.road_segments;

-- Note: Les colonnes chunk_x et chunk_y restent dans road_segments pour compatibilité
-- mais ne sont plus la source de vérité. La table road_chunk_visibility est maintenant
-- la référence pour savoir quels segments sont visibles dans quels chunks.
