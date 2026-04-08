-- Migration: Remap building_type_id values to match new reference document IDs
-- Old IDs (1-55, 1001-1003) → New IDs (100-2002)
--
-- Run SEQUENCE:
--   1. Run this migration
--   2. Run game-seed (populates game.building_types with new IDs)
--   3. Deploy new code

BEGIN;

-- ═══════════════════════════════════════════════════════════════
-- Step 1: Drop ALL FK constraints that reference old type IDs
-- ═══════════════════════════════════════════════════════════════

-- buildings_base → game.building_types
ALTER TABLE buildings.buildings_base DROP CONSTRAINT IF EXISTS buildings_base_building_type_id_fkey;

-- Sub-tables → buildings.*_types lookup tables
ALTER TABLE buildings.manufacturing_workshops DROP CONSTRAINT IF EXISTS manufacturing_workshops_workshop_type_id_fkey;
ALTER TABLE buildings.agriculture DROP CONSTRAINT IF EXISTS agriculture_agriculture_type_id_fkey;
ALTER TABLE buildings.animal_breeding DROP CONSTRAINT IF EXISTS animal_breeding_animal_type_id_fkey;
ALTER TABLE buildings.entertainment DROP CONSTRAINT IF EXISTS entertainment_entertainment_type_id_fkey;
ALTER TABLE buildings.cult DROP CONSTRAINT IF EXISTS cult_cult_type_id_fkey;
ALTER TABLE buildings.commerce DROP CONSTRAINT IF EXISTS commerce_commerce_type_id_fkey;
ALTER TABLE buildings.trees DROP CONSTRAINT IF EXISTS trees_tree_type_id_fkey;

-- actions → buildings.building_types
ALTER TABLE actions.build_building_actions DROP CONSTRAINT IF EXISTS build_building_actions_building_type_id_fkey;

-- ═══════════════════════════════════════════════════════════════
-- Step 2: Clear lookup tables (will be re-seeded by game-seed)
-- ═══════════════════════════════════════════════════════════════

DELETE FROM game.construction_costs;
DELETE FROM game.building_types;

-- Clear and repopulate the buildings schema lookup tables
DELETE FROM buildings.manufacturing_workshop_types;
DELETE FROM buildings.agriculture_types;
DELETE FROM buildings.animal_breeding_types;
DELETE FROM buildings.entertainment_types;
DELETE FROM buildings.cult_types;
DELETE FROM buildings.commerce_types;
-- tree_types stay (Cedar=1, Larch=2, Oak=3 unchanged)

-- ═══════════════════════════════════════════════════════════════
-- Step 3: Migrate sub-table type IDs (existing building instances)
-- ═══════════════════════════════════════════════════════════════

-- Manufacturing workshops
UPDATE buildings.manufacturing_workshops SET workshop_type_id = 240 WHERE workshop_type_id = 1;  -- Blacksmith → Forge
UPDATE buildings.manufacturing_workshops SET workshop_type_id = 230 WHERE workshop_type_id = 2;  -- BlastFurnace → Fonderie
UPDATE buildings.manufacturing_workshops SET workshop_type_id = 230 WHERE workshop_type_id = 3;  -- Bloomery → Fonderie
UPDATE buildings.manufacturing_workshops SET workshop_type_id = 500 WHERE workshop_type_id = 4;  -- CarpenterShop → AtelierCharpentier
UPDATE buildings.manufacturing_workshops SET workshop_type_id = 330 WHERE workshop_type_id = 5;  -- GlassFactory → Verrerie

-- Agriculture
UPDATE buildings.agriculture SET agriculture_type_id = 400 WHERE agriculture_type_id = 10; -- Farm → Ferme

-- Animal breeding
UPDATE buildings.animal_breeding SET animal_type_id = 810 WHERE animal_type_id = 20; -- Cowshed → Etable
UPDATE buildings.animal_breeding SET animal_type_id = 830 WHERE animal_type_id = 21; -- Piggery → Porcherie
UPDATE buildings.animal_breeding SET animal_type_id = 820 WHERE animal_type_id = 22; -- Sheepfold → Bergerie
UPDATE buildings.animal_breeding SET animal_type_id = 840 WHERE animal_type_id = 23; -- Stable → Ecurie

-- Entertainment
UPDATE buildings.entertainment SET entertainment_type_id = 1060 WHERE entertainment_type_id = 30; -- Theater → Theatre

-- Cult
UPDATE buildings.cult SET cult_type_id = 1010 WHERE cult_type_id = 40; -- Temple → LieuDeCulte

-- Commerce
UPDATE buildings.commerce SET commerce_type_id = 420  WHERE commerce_type_id = 50; -- Bakehouse → Cuisine
UPDATE buildings.commerce SET commerce_type_id = 440  WHERE commerce_type_id = 51; -- Brewery → Brasserie
DELETE FROM buildings.commerce WHERE commerce_type_id = 52;                         -- Distillery removed
UPDATE buildings.commerce SET commerce_type_id = 430  WHERE commerce_type_id = 53; -- Slaughterhouse → Abattoir
UPDATE buildings.commerce SET commerce_type_id = 1220 WHERE commerce_type_id = 54; -- IceHouse → Glaciere
UPDATE buildings.commerce SET commerce_type_id = 1000 WHERE commerce_type_id = 55; -- Market → PlaceMarche

-- ═══════════════════════════════════════════════════════════════
-- Step 4: Migrate buildings_base IDs
-- ═══════════════════════════════════════════════════════════════

UPDATE buildings.buildings_base SET building_type_id = 240  WHERE building_type_id = 1;    -- Blacksmith → Forge
UPDATE buildings.buildings_base SET building_type_id = 230  WHERE building_type_id = 2;    -- BlastFurnace → Fonderie
UPDATE buildings.buildings_base SET building_type_id = 230  WHERE building_type_id = 3;    -- Bloomery → Fonderie (merged)
UPDATE buildings.buildings_base SET building_type_id = 500  WHERE building_type_id = 4;    -- CarpenterShop → AtelierCharpentier
UPDATE buildings.buildings_base SET building_type_id = 330  WHERE building_type_id = 5;    -- GlassFactory → Verrerie
UPDATE buildings.buildings_base SET building_type_id = 400  WHERE building_type_id = 10;   -- Farm → Ferme
UPDATE buildings.buildings_base SET building_type_id = 810  WHERE building_type_id = 20;   -- Cowshed → Etable
UPDATE buildings.buildings_base SET building_type_id = 830  WHERE building_type_id = 21;   -- Piggery → Porcherie
UPDATE buildings.buildings_base SET building_type_id = 820  WHERE building_type_id = 22;   -- Sheepfold → Bergerie
UPDATE buildings.buildings_base SET building_type_id = 840  WHERE building_type_id = 23;   -- Stable → Ecurie
UPDATE buildings.buildings_base SET building_type_id = 1060 WHERE building_type_id = 30;   -- Theater → Theatre
UPDATE buildings.buildings_base SET building_type_id = 1010 WHERE building_type_id = 40;   -- Temple → LieuDeCulte
UPDATE buildings.buildings_base SET building_type_id = 420  WHERE building_type_id = 50;   -- Bakehouse → Cuisine
UPDATE buildings.buildings_base SET building_type_id = 440  WHERE building_type_id = 51;   -- Brewery → Brasserie
DELETE FROM buildings.buildings_base WHERE building_type_id = 52;                           -- Distillery removed
UPDATE buildings.buildings_base SET building_type_id = 430  WHERE building_type_id = 53;   -- Slaughterhouse → Abattoir
UPDATE buildings.buildings_base SET building_type_id = 1220 WHERE building_type_id = 54;   -- IceHouse → Glaciere
UPDATE buildings.buildings_base SET building_type_id = 1000 WHERE building_type_id = 55;   -- Market → PlaceMarche
UPDATE buildings.buildings_base SET building_type_id = 2001 WHERE building_type_id = 1001; -- Cedar
UPDATE buildings.buildings_base SET building_type_id = 2002 WHERE building_type_id = 1002; -- Larch
UPDATE buildings.buildings_base SET building_type_id = 2000 WHERE building_type_id = 1003; -- Oak

-- Migrate build_building_actions too
UPDATE actions.build_building_actions SET building_type_id = 240  WHERE building_type_id = 1;
UPDATE actions.build_building_actions SET building_type_id = 230  WHERE building_type_id = 2;
UPDATE actions.build_building_actions SET building_type_id = 230  WHERE building_type_id = 3;
UPDATE actions.build_building_actions SET building_type_id = 500  WHERE building_type_id = 4;
UPDATE actions.build_building_actions SET building_type_id = 330  WHERE building_type_id = 5;
UPDATE actions.build_building_actions SET building_type_id = 400  WHERE building_type_id = 10;
UPDATE actions.build_building_actions SET building_type_id = 810  WHERE building_type_id = 20;
UPDATE actions.build_building_actions SET building_type_id = 830  WHERE building_type_id = 21;
UPDATE actions.build_building_actions SET building_type_id = 820  WHERE building_type_id = 22;
UPDATE actions.build_building_actions SET building_type_id = 840  WHERE building_type_id = 23;
UPDATE actions.build_building_actions SET building_type_id = 1060 WHERE building_type_id = 30;
UPDATE actions.build_building_actions SET building_type_id = 1010 WHERE building_type_id = 40;
UPDATE actions.build_building_actions SET building_type_id = 420  WHERE building_type_id = 50;
UPDATE actions.build_building_actions SET building_type_id = 440  WHERE building_type_id = 51;
DELETE FROM actions.build_building_actions WHERE building_type_id = 52;
UPDATE actions.build_building_actions SET building_type_id = 430  WHERE building_type_id = 53;
UPDATE actions.build_building_actions SET building_type_id = 1220 WHERE building_type_id = 54;
UPDATE actions.build_building_actions SET building_type_id = 1000 WHERE building_type_id = 55;

COMMIT;

-- ═══════════════════════════════════════════════════════════════
-- Step 5: After running game-seed, the FK constraints are NOT
-- re-added. The Rust enums are the source of truth for type
-- validation. The lookup tables are kept for game-seed data only.
-- ═══════════════════════════════════════════════════════════════
