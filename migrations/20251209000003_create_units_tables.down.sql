-- Drop units main tables in reverse order

-- Drop triggers
DROP TRIGGER IF EXISTS update_unit_automated_actions_updated_at ON units.unit_automated_actions;
DROP TRIGGER IF EXISTS update_unit_inventory_updated_at ON units.unit_inventory;
DROP TRIGGER IF EXISTS update_unit_skills_updated_at ON units.unit_skills;
DROP TRIGGER IF EXISTS update_unit_derived_stats_updated_at ON units.unit_derived_stats;
DROP TRIGGER IF EXISTS update_unit_base_stats_updated_at ON units.unit_base_stats;
DROP TRIGGER IF EXISTS update_units_updated_at ON units.units;

-- Drop function
DROP FUNCTION IF EXISTS units.update_updated_at_column();

-- Drop tables
DROP TABLE IF EXISTS units.unit_consumption_demands CASCADE;
DROP TABLE IF EXISTS units.unit_automated_actions CASCADE;
DROP TABLE IF EXISTS units.unit_equipment CASCADE;
DROP TABLE IF EXISTS units.unit_inventory CASCADE;
DROP TABLE IF EXISTS units.unit_skills CASCADE;
DROP TABLE IF EXISTS units.unit_derived_stats CASCADE;
DROP TABLE IF EXISTS units.unit_base_stats CASCADE;
DROP TABLE IF EXISTS units.units CASCADE;
