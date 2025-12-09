-- Update units tables to use resources.items instead of units.items

-- Supprimer units.items si elle existe (créée par erreur dans 20251209000002)
DROP TABLE IF EXISTS units.item_stat_modifiers CASCADE;
DROP TABLE IF EXISTS units.items CASCADE;
DROP TABLE IF EXISTS units.equipment_slots CASCADE;
DROP TABLE IF EXISTS units.item_types CASCADE;

-- Modifier units.unit_inventory pour référencer resources.items
ALTER TABLE units.unit_inventory
    DROP CONSTRAINT IF EXISTS unit_inventory_item_id_fkey,
    ADD CONSTRAINT unit_inventory_item_id_fkey
        FOREIGN KEY (item_id) REFERENCES resources.items(id);

-- Modifier units.unit_equipment pour référencer resources.items et resources.equipment_slots
ALTER TABLE units.unit_equipment
    DROP CONSTRAINT IF EXISTS unit_equipment_item_id_fkey,
    ADD CONSTRAINT unit_equipment_item_id_fkey
        FOREIGN KEY (item_id) REFERENCES resources.items(id);

ALTER TABLE units.unit_equipment
    DROP CONSTRAINT IF EXISTS unit_equipment_equipment_slot_id_fkey,
    ADD CONSTRAINT unit_equipment_equipment_slot_id_fkey
        FOREIGN KEY (equipment_slot_id) REFERENCES resources.equipment_slots(id);

-- Modifier units.unit_consumption_demands pour référencer resources.items
ALTER TABLE units.unit_consumption_demands
    DROP CONSTRAINT IF EXISTS unit_consumption_demands_item_id_fkey,
    ADD CONSTRAINT unit_consumption_demands_item_id_fkey
        FOREIGN KEY (item_id) REFERENCES resources.items(id);

COMMENT ON CONSTRAINT unit_inventory_item_id_fkey ON units.unit_inventory
    IS 'References items from resources.items table';

COMMENT ON CONSTRAINT unit_equipment_item_id_fkey ON units.unit_equipment
    IS 'References items from resources.items table';

COMMENT ON CONSTRAINT unit_consumption_demands_item_id_fkey ON units.unit_consumption_demands
    IS 'References items from resources.items table';
