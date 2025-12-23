-- Revert tree building_type_ids back to incorrect values (for rollback)
-- This assumes trees were incorrectly using building_type_id = 1 before

-- Revert all trees back to building_type_id = 1 (incorrect but original state)
UPDATE buildings.buildings_base
SET building_type_id = 1
WHERE category_id = 1 AND id IN (
    SELECT building_id FROM buildings.trees
);

-- Remove tree entries from building_types
DELETE FROM buildings.building_types WHERE id IN (1001, 1002, 1003);
