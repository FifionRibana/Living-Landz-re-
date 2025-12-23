-- Fix all constructed buildings to use correct building_type_id based on their specific type
-- This corrects buildings that were created with incorrect building_type_id values

-- Entertainment buildings (category 6) → Theater (30)
UPDATE buildings.buildings_base
SET building_type_id = 30
WHERE category_id = 6
  AND id IN (
    SELECT building_id FROM buildings.entertainment WHERE entertainment_type_id = 30
  );

-- Agriculture buildings (category 7) → Farm (10)
UPDATE buildings.buildings_base
SET building_type_id = 10
WHERE category_id = 7
  AND id IN (
    SELECT building_id FROM buildings.agriculture WHERE agriculture_type_id = 10
  );

-- Animal Breeding buildings (category 8)
-- Cowshed (20)
UPDATE buildings.buildings_base
SET building_type_id = 20
WHERE category_id = 8
  AND id IN (
    SELECT building_id FROM buildings.animal_breeding WHERE animal_type_id = 20
  );

-- Piggery (21)
UPDATE buildings.buildings_base
SET building_type_id = 21
WHERE category_id = 8
  AND id IN (
    SELECT building_id FROM buildings.animal_breeding WHERE animal_type_id = 21
  );

-- Sheepfold (22)
UPDATE buildings.buildings_base
SET building_type_id = 22
WHERE category_id = 8
  AND id IN (
    SELECT building_id FROM buildings.animal_breeding WHERE animal_type_id = 22
  );

-- Stable (23)
UPDATE buildings.buildings_base
SET building_type_id = 23
WHERE category_id = 8
  AND id IN (
    SELECT building_id FROM buildings.animal_breeding WHERE animal_type_id = 23
  );

-- Cult buildings (category 3) → Temple (40)
UPDATE buildings.buildings_base
SET building_type_id = 40
WHERE category_id = 3
  AND id IN (
    SELECT building_id FROM buildings.cult WHERE cult_type_id = 40
  );

-- Commerce buildings (category 11)
-- Bakehouse (50)
UPDATE buildings.buildings_base
SET building_type_id = 50
WHERE category_id = 11
  AND id IN (
    SELECT building_id FROM buildings.commerce WHERE commerce_type_id = 50
  );

-- Brewery (51)
UPDATE buildings.buildings_base
SET building_type_id = 51
WHERE category_id = 11
  AND id IN (
    SELECT building_id FROM buildings.commerce WHERE commerce_type_id = 51
  );

-- Distillery (52)
UPDATE buildings.buildings_base
SET building_type_id = 52
WHERE category_id = 11
  AND id IN (
    SELECT building_id FROM buildings.commerce WHERE commerce_type_id = 52
  );

-- Slaughterhouse (53)
UPDATE buildings.buildings_base
SET building_type_id = 53
WHERE category_id = 11
  AND id IN (
    SELECT building_id FROM buildings.commerce WHERE commerce_type_id = 53
  );

-- IceHouse (54)
UPDATE buildings.buildings_base
SET building_type_id = 54
WHERE category_id = 11
  AND id IN (
    SELECT building_id FROM buildings.commerce WHERE commerce_type_id = 54
  );

-- Market (55)
UPDATE buildings.buildings_base
SET building_type_id = 55
WHERE category_id = 11
  AND id IN (
    SELECT building_id FROM buildings.commerce WHERE commerce_type_id = 55
  );

-- Manufacturing Workshops (category 5)
-- Blacksmith (1)
UPDATE buildings.buildings_base
SET building_type_id = 1
WHERE category_id = 5
  AND id IN (
    SELECT building_id FROM buildings.manufacturing_workshops WHERE workshop_type_id = 1
  );

-- BlastFurnace (2)
UPDATE buildings.buildings_base
SET building_type_id = 2
WHERE category_id = 5
  AND id IN (
    SELECT building_id FROM buildings.manufacturing_workshops WHERE workshop_type_id = 2
  );

-- Bloomery (3)
UPDATE buildings.buildings_base
SET building_type_id = 3
WHERE category_id = 5
  AND id IN (
    SELECT building_id FROM buildings.manufacturing_workshops WHERE workshop_type_id = 3
  );

-- CarpenterShop (4)
UPDATE buildings.buildings_base
SET building_type_id = 4
WHERE category_id = 5
  AND id IN (
    SELECT building_id FROM buildings.manufacturing_workshops WHERE workshop_type_id = 4
  );

-- GlassFactory (5)
UPDATE buildings.buildings_base
SET building_type_id = 5
WHERE category_id = 5
  AND id IN (
    SELECT building_id FROM buildings.manufacturing_workshops WHERE workshop_type_id = 5
  );
