-- Create recipe system for crafting

-- Recipes (recettes de craft)
CREATE TABLE resources.recipes (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL UNIQUE,
    description TEXT,

    -- Résultat principal
    result_item_id INT NOT NULL REFERENCES resources.items(id),
    result_quantity INT NOT NULL DEFAULT 1,

    -- Skill requis
    required_skill_id SMALLINT, -- NULL = pas de skill requis
    required_skill_level INT DEFAULT 1,

    -- Temps de craft
    craft_duration_seconds INT NOT NULL DEFAULT 10,

    -- Station requise (optionnel)
    required_building_type_id SMALLINT, -- NULL = craftable partout

    archived BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),

    CONSTRAINT chk_result_quantity CHECK (result_quantity > 0),
    CONSTRAINT chk_craft_duration CHECK (craft_duration_seconds > 0)
);

CREATE INDEX idx_recipes_result_item ON resources.recipes(result_item_id);
CREATE INDEX idx_recipes_skill ON resources.recipes(required_skill_id) WHERE required_skill_id IS NOT NULL;

COMMENT ON TABLE resources.recipes IS 'Crafting recipes for creating items';
COMMENT ON COLUMN resources.recipes.required_building_type_id IS 'Building type ID from buildings.building_types, NULL if can craft anywhere';

-- Recipe ingredients (ingrédients des recettes)
CREATE TABLE resources.recipe_ingredients (
    recipe_id INT NOT NULL REFERENCES resources.recipes(id) ON DELETE CASCADE,
    item_id INT NOT NULL REFERENCES resources.items(id),
    quantity INT NOT NULL,
    PRIMARY KEY (recipe_id, item_id),
    CONSTRAINT chk_ingredient_quantity CHECK (quantity > 0)
);

CREATE INDEX idx_recipe_ingredients_recipe ON resources.recipe_ingredients(recipe_id);
CREATE INDEX idx_recipe_ingredients_item ON resources.recipe_ingredients(item_id);

COMMENT ON TABLE resources.recipe_ingredients IS 'Ingredients required for each recipe';

-- Exemples de recettes
INSERT INTO resources.recipes
    (id, name, description, result_item_id, result_quantity, required_skill_id, required_skill_level, craft_duration_seconds)
VALUES
    -- Bread (Baking skill = 42)
    (1, 'Bake Bread', 'Bake bread from wheat', 10, 2, 42, 1, 30),

    -- Cooked Meat (Cooking skill = 41)
    (2, 'Cook Meat', 'Cook raw meat', 13, 1, 41, 1, 20),

    -- Iron Pickaxe (Blacksmithing skill = 5)
    (3, 'Forge Iron Pickaxe', 'Craft an iron pickaxe', 20, 1, 5, 3, 60),

    -- Iron Axe (Blacksmithing skill = 5)
    (4, 'Forge Iron Axe', 'Craft an iron axe', 21, 1, 5, 2, 50),

    -- Fishing Rod (Crafting skill = 30)
    (5, 'Craft Fishing Rod', 'Craft a fishing rod', 22, 1, 30, 1, 40),

    -- Hammer (Blacksmithing skill = 5)
    (6, 'Forge Hammer', 'Craft a hammer', 23, 1, 5, 2, 45),

    -- Iron Sword (Blacksmithing skill = 5)
    (7, 'Forge Iron Sword', 'Craft an iron sword', 30, 1, 5, 5, 90),

    -- Wooden Bow (Crafting skill = 30)
    (8, 'Craft Wooden Bow', 'Craft a wooden bow', 31, 1, 30, 3, 50),

    -- Iron Shield (Blacksmithing skill = 5)
    (9, 'Forge Iron Shield', 'Craft an iron shield', 32, 1, 5, 4, 75),

    -- Leather Helmet (Crafting skill = 30)
    (10, 'Craft Leather Helmet', 'Craft leather helmet', 40, 1, 30, 2, 35),

    -- Leather Chest (Crafting skill = 30)
    (11, 'Craft Leather Chest', 'Craft leather chest armor', 41, 1, 30, 3, 45),

    -- Small Backpack (Crafting skill = 30)
    (12, 'Craft Small Backpack', 'Craft a small backpack', 50, 1, 30, 1, 30),

    -- Large Backpack (Crafting skill = 30)
    (13, 'Craft Large Backpack', 'Craft a large backpack', 51, 1, 30, 4, 60);

-- Ingrédients des recettes
INSERT INTO resources.recipe_ingredients (recipe_id, item_id, quantity) VALUES
    -- Bread: 2 Wheat
    (1, 4, 2),

    -- Cooked Meat: 1 Meat
    (2, 12, 1),

    -- Iron Pickaxe: 2 Iron Ore, 1 Wood, 1 Coal
    (3, 3, 2),
    (3, 1, 1),
    (3, 5, 1),

    -- Iron Axe: 2 Iron Ore, 1 Wood
    (4, 3, 2),
    (4, 1, 1),

    -- Fishing Rod: 2 Wood
    (5, 1, 2),

    -- Hammer: 2 Iron Ore, 1 Wood
    (6, 3, 2),
    (6, 1, 1),

    -- Iron Sword: 3 Iron Ore, 1 Wood, 1 Coal
    (7, 3, 3),
    (7, 1, 1),
    (7, 5, 1),

    -- Wooden Bow: 3 Wood
    (8, 1, 3),

    -- Iron Shield: 4 Iron Ore, 1 Wood, 1 Coal
    (9, 3, 4),
    (9, 1, 1),
    (9, 5, 1),

    -- Leather Helmet: 1 Wood (simplifié - devrait être du cuir)
    (10, 1, 1),

    -- Leather Chest: 2 Wood (simplifié)
    (11, 1, 2),

    -- Small Backpack: 2 Wood
    (12, 1, 2),

    -- Large Backpack: 4 Wood
    (13, 1, 4);

-- Mettre à jour la séquence
SELECT setval('resources.recipes_id_seq', GREATEST(50, (SELECT MAX(id) FROM resources.recipes)));
