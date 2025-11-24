-- Create actions tables

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
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_unique_cell_action
    ON actions.scheduled_actions(cell_q, cell_r)
    WHERE status_id IN (1, 2);

CREATE INDEX idx_completion_time ON actions.scheduled_actions(completion_time);

CREATE INDEX idx_chunk_commands ON actions.scheduled_actions(chunk_x, chunk_y)
    WHERE status_id IN (1, 2);

CREATE INDEX idx_player_commands ON actions.scheduled_actions(player_id)
    WHERE status_id IN (1, 2);

-- Specific action tables

CREATE TABLE actions.build_building_actions (
    action_id BIGINT PRIMARY KEY REFERENCES actions.scheduled_actions(id) ON DELETE CASCADE,
    building_type_id INT NOT NULL REFERENCES buildings.building_types(id)
);

CREATE TABLE actions.build_road_actions (
    action_id BIGINT PRIMARY KEY REFERENCES actions.scheduled_actions(id) ON DELETE CASCADE
);

CREATE TABLE actions.move_unit_actions (
    action_id BIGINT PRIMARY KEY REFERENCES actions.scheduled_actions(id) ON DELETE CASCADE,
    unit_id BIGINT NOT NULL,
    target_q INT NOT NULL,
    target_r INT NOT NULL
);

CREATE TABLE actions.send_message_actions (
    action_id BIGINT PRIMARY KEY REFERENCES actions.scheduled_actions(id) ON DELETE CASCADE,
    message_content TEXT NOT NULL
);

-- Table associative pour les receivers
CREATE TABLE actions.send_message_receivers (
    id BIGSERIAL PRIMARY KEY,
    action_id BIGINT NOT NULL REFERENCES actions.send_message_actions(action_id) ON DELETE CASCADE,
    receiver_id BIGINT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE actions.harvest_resource_actions (
    action_id BIGINT PRIMARY KEY REFERENCES actions.scheduled_actions(id) ON DELETE CASCADE,
    resource_type_id INT NOT NULL REFERENCES resources.resource_types(id)
);

CREATE TABLE actions.craft_resource_actions (
    action_id BIGINT PRIMARY KEY REFERENCES actions.scheduled_actions(id) ON DELETE CASCADE,
    recipe_id VARCHAR NOT NULL,
    quantity INT NOT NULL
);

-- Indices
CREATE INDEX idx_building_type_id ON actions.build_building_actions(building_type_id);
CREATE INDEX idx_resource_type_id ON actions.harvest_resource_actions(resource_type_id);
CREATE INDEX idx_message_receivers ON actions.send_message_receivers(action_id);
CREATE INDEX idx_receiver_messages ON actions.send_message_receivers(receiver_id);
