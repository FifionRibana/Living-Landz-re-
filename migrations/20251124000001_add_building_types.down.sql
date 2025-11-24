-- Add down migration script here

-- Remove all building types
DELETE FROM buildings.building_types WHERE id IN (
    1, 2, 3, 4, 5,      -- ManufacturingWorkshops
    10,                  -- Agriculture
    20, 21, 22, 23,     -- AnimalBreeding
    30,                  -- Entertainment
    40,                  -- Cult
    50, 51, 52, 53, 54, 55  -- Commerce
);

-- Remove Commerce category
DELETE FROM buildings.building_categories WHERE id = 11;
