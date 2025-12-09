-- Add start_cell and end_cell to build_road_actions table

ALTER TABLE actions.build_road_actions
ADD COLUMN start_q INT NOT NULL DEFAULT 0,
ADD COLUMN start_r INT NOT NULL DEFAULT 0,
ADD COLUMN end_q INT NOT NULL DEFAULT 0,
ADD COLUMN end_r INT NOT NULL DEFAULT 0;

-- Remove defaults after initial migration
ALTER TABLE actions.build_road_actions
ALTER COLUMN start_q DROP DEFAULT,
ALTER COLUMN start_r DROP DEFAULT,
ALTER COLUMN end_q DROP DEFAULT,
ALTER COLUMN end_r DROP DEFAULT;
