-- Revert units tables to not reference resources.items

-- Note: This migration cannot fully revert because it would require recreating units.items
-- which was incorrectly created. The down migration removes the foreign keys.

ALTER TABLE units.unit_consumption_demands
    DROP CONSTRAINT IF EXISTS unit_consumption_demands_item_id_fkey;

ALTER TABLE units.unit_equipment
    DROP CONSTRAINT IF EXISTS unit_equipment_equipment_slot_id_fkey,
    DROP CONSTRAINT IF EXISTS unit_equipment_item_id_fkey;

ALTER TABLE units.unit_inventory
    DROP CONSTRAINT IF EXISTS unit_inventory_item_id_fkey;
