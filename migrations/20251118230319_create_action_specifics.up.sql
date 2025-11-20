-- Add up migration script here
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
    created_at TIMESTAMP DEFAULT NOW()
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