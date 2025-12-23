-- Fix tree building_type_ids to use proper IDs in 1000+ range
-- Natural category (trees) should use 1000+ range to avoid conflicts with constructed buildings

-- Step 1: First, create entries with correct IDs if they don't already exist
-- Use DO NOTHING to avoid conflicts if IDs already exist
INSERT INTO buildings.building_types (id, name, category_id, specific_type_id, description)
SELECT 1001, 'Cedar_new', 1, 1, 'Cedar tree'
WHERE NOT EXISTS (SELECT 1 FROM buildings.building_types WHERE id = 1001);

INSERT INTO buildings.building_types (id, name, category_id, specific_type_id, description)
SELECT 1002, 'Larch_new', 1, 1, 'Larch tree'
WHERE NOT EXISTS (SELECT 1 FROM buildings.building_types WHERE id = 1002);

INSERT INTO buildings.building_types (id, name, category_id, specific_type_id, description)
SELECT 1003, 'Oak_new', 1, 1, 'Oak tree'
WHERE NOT EXISTS (SELECT 1 FROM buildings.building_types WHERE id = 1003);

-- Step 2: Update buildings_base to point to new IDs based on tree_type_id
-- Cedar trees (tree_type_id = 1 in buildings.trees)
UPDATE buildings.buildings_base
SET building_type_id = 1001
WHERE id IN (
    SELECT building_id
    FROM buildings.trees
    WHERE tree_type_id = 1
);

-- Larch trees (tree_type_id = 2 in buildings.trees)
UPDATE buildings.buildings_base
SET building_type_id = 1002
WHERE id IN (
    SELECT building_id
    FROM buildings.trees
    WHERE tree_type_id = 2
);

-- Oak trees (tree_type_id = 3 in buildings.trees)
UPDATE buildings.buildings_base
SET building_type_id = 1003
WHERE id IN (
    SELECT building_id
    FROM buildings.trees
    WHERE tree_type_id = 3
);

-- Step 4: Delete old entries with the same names but different IDs (if any)
DELETE FROM buildings.building_types
WHERE name IN ('Cedar', 'Larch', 'Oak')
  AND category_id = 1
  AND specific_type_id = 1
  AND id NOT IN (1001, 1002, 1003);
  
-- Step 3: Now that all buildings_base are updated, rename the entries to their correct names
UPDATE buildings.building_types
SET name = 'Cedar'
WHERE id = 1001 AND name = 'Cedar_new';

UPDATE buildings.building_types
SET name = 'Larch'
WHERE id = 1002 AND name = 'Larch_new';

UPDATE buildings.building_types
SET name = 'Oak'
WHERE id = 1003 AND name = 'Oak_new';


-- Update the sequence to ensure no conflicts
SELECT setval('buildings.building_types_id_seq', GREATEST(1003, (SELECT MAX(id) FROM buildings.building_types)));
