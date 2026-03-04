-- Add TrainUnit to action type lookups

INSERT INTO actions.action_types (id, name) VALUES (7, 'TrainUnit')
    ON CONFLICT (id) DO NOTHING;

INSERT INTO actions.action_specific_types (id, name) VALUES (7, 'TrainUnit')
    ON CONFLICT (id) DO NOTHING;

-- Specific table for training actions

CREATE TABLE IF NOT EXISTS actions.train_unit_actions (
    action_id BIGINT PRIMARY KEY REFERENCES actions.scheduled_actions(id) ON DELETE CASCADE,
    unit_id BIGINT NOT NULL,
    target_profession_id SMALLINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_train_unit_id ON actions.train_unit_actions(unit_id);
