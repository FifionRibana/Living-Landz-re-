-- Add entry for Unknown/Tree buildings in building_types
-- This allows trees and unknown buildings to reference building_types with id=0

INSERT INTO buildings.building_types (id, name, category_id, specific_type_id, description)
VALUES (0, 'Unknown', 0, 0, 'Unknown or natural formations')
ON CONFLICT (id) DO NOTHING;
