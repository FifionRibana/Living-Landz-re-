-- Create item instances for quality and perishability tracking

-- Item instances (instances d'items avec qualité et périssabilité)
CREATE TABLE resources.item_instances (
    id BIGSERIAL PRIMARY KEY,
    item_id INT NOT NULL REFERENCES resources.items(id),

    -- Qualité (0.0 - 1.0, 1.0 = perfect quality)
    quality DECIMAL(3, 2) NOT NULL DEFAULT 1.0,

    -- Périssabilité (pour les items périssables)
    current_decay DECIMAL(3, 2) DEFAULT 0.0, -- 0.0 = fresh, 1.0 = complètement pourri
    last_decay_update TIMESTAMPTZ, -- Dernière mise à jour du decay

    -- Propriétaire actuel (optionnel, NULL si dans le monde)
    owner_unit_id BIGINT, -- REFERENCES units.units(id) - foreign key ajoutée plus tard

    -- Position dans le monde (si pas dans un inventaire)
    world_cell_q INT,
    world_cell_r INT,
    world_chunk_x INT,
    world_chunk_y INT,

    created_at TIMESTAMPTZ DEFAULT NOW(),

    CONSTRAINT chk_quality CHECK (quality >= 0.0 AND quality <= 1.0),
    CONSTRAINT chk_current_decay CHECK (current_decay >= 0.0 AND current_decay <= 1.0)
);

CREATE INDEX idx_item_instances_item ON resources.item_instances(item_id);
CREATE INDEX idx_item_instances_owner ON resources.item_instances(owner_unit_id) WHERE owner_unit_id IS NOT NULL;
CREATE INDEX idx_item_instances_world_pos ON resources.item_instances(world_chunk_x, world_chunk_y, world_cell_q, world_cell_r)
    WHERE owner_unit_id IS NULL;
CREATE INDEX idx_item_instances_perishable ON resources.item_instances(current_decay, last_decay_update)
    WHERE current_decay > 0;

COMMENT ON TABLE resources.item_instances IS 'Individual item instances with quality and decay tracking';
COMMENT ON COLUMN resources.item_instances.quality IS 'Quality of the item, 0.0 to 1.0, affects stats and price';
COMMENT ON COLUMN resources.item_instances.current_decay IS 'Current decay level, 0.0 = fresh, 1.0 = completely rotten';
COMMENT ON COLUMN resources.item_instances.owner_unit_id IS 'Unit that owns this item, NULL if in the world';

-- Fonction pour mettre à jour le decay automatiquement
CREATE OR REPLACE FUNCTION resources.update_item_decay()
RETURNS TRIGGER AS $$
DECLARE
    item_record RECORD;
    time_diff_seconds BIGINT;
    time_diff_days DECIMAL;
    decay_amount DECIMAL;
BEGIN
    -- Récupérer les infos de l'item
    SELECT is_perishable, base_decay_rate_per_day
    INTO item_record
    FROM resources.items
    WHERE id = NEW.item_id;

    -- Si l'item est périssable et a un decay rate
    IF item_record.is_perishable AND item_record.base_decay_rate_per_day > 0 THEN
        -- Si c'est une nouvelle instance ou pas de dernière mise à jour
        IF NEW.last_decay_update IS NULL THEN
            NEW.last_decay_update := NOW();
            NEW.current_decay := 0.0;
        ELSE
            -- Calculer le temps écoulé en jours
            time_diff_seconds := EXTRACT(EPOCH FROM (NOW() - NEW.last_decay_update));
            time_diff_days := time_diff_seconds / 86400.0;

            -- Calculer le decay
            decay_amount := item_record.base_decay_rate_per_day * time_diff_days;
            NEW.current_decay := LEAST(1.0, NEW.current_decay + decay_amount);
            NEW.last_decay_update := NOW();
        END IF;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger pour mettre à jour le decay avant l'insertion/mise à jour
CREATE TRIGGER trigger_update_item_decay
    BEFORE INSERT OR UPDATE ON resources.item_instances
    FOR EACH ROW
    EXECUTE FUNCTION resources.update_item_decay();

COMMENT ON FUNCTION resources.update_item_decay() IS 'Automatically updates item decay based on time elapsed';
