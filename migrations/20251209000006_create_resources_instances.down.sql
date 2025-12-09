-- Drop item instances

DROP TRIGGER IF EXISTS trigger_update_item_decay ON resources.item_instances;
DROP FUNCTION IF EXISTS resources.update_item_decay();
DROP TABLE IF EXISTS resources.item_instances CASCADE;
