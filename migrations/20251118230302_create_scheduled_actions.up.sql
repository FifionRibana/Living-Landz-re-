-- Add up migration script here
CREATE TABLE actions.scheduled_actions (
    id BIGSERIAL PRIMARY KEY,
    player_id BIGINT NOT NULL,
    cell_q INT NOT NULL,
    cell_r INT NOT NULL,
    chunk_x INT NOT NULL,
    chunk_y INT NOT NULL,
    action_type_id SMALLINT NOT NULL REFERENCES actions.action_types(id),
    action_specific_type_id SMALLINT NOT NULL REFERENCES actions.action_specific_types(id),
    start_time BIGINT NOT NULL,
    duration_ms BIGINT NOT NULL,
    completion_time BIGINT NOT NULL,
    status_id SMALLINT NOT NULL REFERENCES actions.action_statuses(id) DEFAULT 1,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_unique_cell_action 
    ON actions.scheduled_actions(cell_q, cell_r) 
    WHERE status_id IN (1, 2);

CREATE INDEX idx_completion_time ON actions.scheduled_actions(completion_time);

CREATE INDEX idx_chunk_commands ON actions.scheduled_actions(chunk_x, chunk_y) 
    WHERE status_id IN (1, 2);

CREATE INDEX idx_player_commands ON actions.scheduled_actions(player_id) 
    WHERE status_id IN (1, 2);