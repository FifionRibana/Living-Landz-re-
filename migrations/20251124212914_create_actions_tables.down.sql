-- Revert actions tables

DROP INDEX IF EXISTS idx_receiver_messages;
DROP INDEX IF EXISTS idx_message_receivers;
DROP INDEX IF EXISTS idx_resource_type_id;
DROP INDEX IF EXISTS idx_building_type_id;

DROP TABLE IF EXISTS actions.craft_resource_actions;
DROP TABLE IF EXISTS actions.harvest_resource_actions;
DROP TABLE IF EXISTS actions.send_message_receivers;
DROP TABLE IF EXISTS actions.send_message_actions;
DROP TABLE IF EXISTS actions.move_unit_actions;
DROP TABLE IF EXISTS actions.build_road_actions;
DROP TABLE IF EXISTS actions.build_building_actions;

DROP INDEX IF EXISTS idx_player_commands;
DROP INDEX IF EXISTS idx_chunk_commands;
DROP INDEX IF EXISTS idx_completion_time;
DROP INDEX IF EXISTS idx_unique_cell_action;

DROP TABLE IF EXISTS actions.scheduled_actions;
