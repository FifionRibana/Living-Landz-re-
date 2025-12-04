-- Create road tables

-- Table pour les segments de route
CREATE TABLE terrain.road_segments (
    id BIGSERIAL PRIMARY KEY,

    -- Nœuds de départ et d'arrivée (coordonnées axiales hexagonales)
    start_q INT NOT NULL,
    start_r INT NOT NULL,
    end_q INT NOT NULL,
    end_r INT NOT NULL,

    -- Polyline encodée (points intermédiaires pour courbes)
    -- Stocké comme BYTEA pour efficacité (bincode encoding de Vec<[f32; 2]>)
    points BYTEA NOT NULL,

    -- Importance du segment (0=sentier, 1=chemin, 2=route, 3=route principale)
    importance SMALLINT NOT NULL CHECK (importance >= 0 AND importance <= 3),

    -- Chunk(s) auquel appartient ce segment
    -- Un segment peut traverser plusieurs chunks, on stocke le chunk principal (du start_node)
    chunk_x INT NOT NULL,
    chunk_y INT NOT NULL,

    -- Métadonnées
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,

    -- Un segment entre deux nœuds doit être unique (bidirectionnel)
    CONSTRAINT unique_segment UNIQUE (start_q, start_r, end_q, end_r),

    -- Référence vers les cellules hexagonales
    FOREIGN KEY (start_q, start_r) REFERENCES terrain.cells(q, r) ON DELETE CASCADE,
    FOREIGN KEY (end_q, end_r) REFERENCES terrain.cells(q, r) ON DELETE CASCADE
);

-- Index pour requêtes rapides par chunk
CREATE INDEX idx_road_segments_chunk ON terrain.road_segments(chunk_x, chunk_y);

-- Index pour requêtes par nœud de départ
CREATE INDEX idx_road_segments_start ON terrain.road_segments(start_q, start_r);

-- Index pour requêtes par nœud d'arrivée
CREATE INDEX idx_road_segments_end ON terrain.road_segments(end_q, end_r);

-- Index composite pour retrouver tous les segments connectés à un nœud (dans les deux directions)
CREATE INDEX idx_road_segments_connections ON terrain.road_segments(start_q, start_r, end_q, end_r);

-- Table pour les données SDF de routes pré-calculées par chunk (cache optionnel)
CREATE TABLE terrain.road_chunk_cache (
    chunk_x INT NOT NULL,
    chunk_y INT NOT NULL,

    -- SDF data encodé (même format que terrain SDF)
    sdf_data BYTEA NOT NULL,

    -- Résolution de la texture SDF
    sdf_width INT NOT NULL,
    sdf_height INT NOT NULL,

    -- Hash des segments pour invalidation du cache
    -- MD5 hash de la concaténation des IDs de segments triés
    segments_hash BYTEA NOT NULL,

    -- Métadonnées
    generated_at BIGINT NOT NULL,

    PRIMARY KEY (chunk_x, chunk_y)
);

-- Index sur la date de génération pour nettoyage du cache
CREATE INDEX idx_road_cache_generated ON terrain.road_chunk_cache(generated_at);
