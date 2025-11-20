-- Add down migration script here
DROP TABLE IF EXISTS actions.action_specific_types CASCADE;
DROP TABLE IF EXISTS actions.action_types CASCADE;
DROP TABLE IF EXISTS actions.action_statuses CASCADE;