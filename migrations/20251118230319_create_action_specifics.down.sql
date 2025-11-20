-- Add down migration script here
DROP TABLE IF EXISTS actions.craft_resource_actions CASCADE;
DROP TABLE IF EXISTS actions.harvest_resource_actions CASCADE;
DROP TABLE IF EXISTS actions.send_message_receivers CASCADE;
DROP TABLE IF EXISTS actions.send_message_actions CASCADE;
DROP TABLE IF EXISTS actions.move_unit_actions CASCADE;
DROP TABLE IF EXISTS actions.build_road_actions CASCADE;
DROP TABLE IF EXISTS actions.build_building_actions CASCADE;