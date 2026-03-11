"""Validate referential integrity of seed data after slug resolution.

By the time data reaches this module, all slugs have been resolved
to numeric IDs by the loader. This validates that the IDs are
consistent across all entities.
"""

import logging

from game_seed.loader import SeedData

logger = logging.getLogger(__name__)


def validate_seed_data(data: SeedData) -> list[str]:
    """Validate all cross-references in the resolved seed data.

    Returns:
        A list of human-readable error strings. Empty means valid.
    """
    errors: list[str] = []

    item_ids = {i.id for i in data.items}
    recipe_ids = {r.id for r in data.recipes}
    building_type_ids = {bt.id for bt in data.building_types}
    resource_cat_ids = {rc.id for rc in data.resource_categories}
    resource_spec_ids = {rs.id for rs in data.resource_specific_types}
    building_cat_ids = {bc.id for bc in data.building_categories}
    building_spec_ids = {bs.id for bs in data.building_specific_types}
    item_type_ids = {it.id for it in data.item_types}
    equipment_slot_ids = {es.id for es in data.equipment_slots}
    profession_ids = {p.id for p in data.professions}
    skill_ids = {s.id for s in data.skills}

    # Duplicate IDs
    _check_duplicates(errors, "items", [i.id for i in data.items])
    _check_duplicates(errors, "items (slugs)", [i.slug for i in data.items])
    _check_duplicates(errors, "recipes", [r.id for r in data.recipes])
    _check_duplicates(errors, "recipes (slugs)", [r.slug for r in data.recipes])
    _check_duplicates(errors, "building_types", [b.id for b in data.building_types])
    _check_duplicates(
        errors, "building_types (slugs)", [b.slug for b in data.building_types]
    )
    _check_duplicates(errors, "harvest_yields", [h.id for h in data.harvest_yields])

    # Items
    for item in data.items:
        if item_type_ids and item.item_type_id not in item_type_ids:
            errors.append(
                f"Item '{item.slug}': item_type_id={item.item_type_id} unknown"
            )
        if item.category_id is not None and resource_cat_ids:
            if item.category_id not in resource_cat_ids:
                errors.append(
                    f"Item '{item.slug}': category_id={item.category_id} unknown"
                )
        if item.equipment_slot_id is not None and equipment_slot_ids:
            if item.equipment_slot_id not in equipment_slot_ids:
                errors.append(
                    f"Item '{item.slug}': "
                    f"equipment_slot_id={item.equipment_slot_id} unknown"
                )

    # Recipes
    for recipe in data.recipes:
        if recipe.result_item_id not in item_ids:
            errors.append(
                f"Recipe '{recipe.slug}': result_item_id={recipe.result_item_id} "
                f"not in items"
            )
        if recipe.required_skill_id is not None:
            if skill_ids and recipe.required_skill_id not in skill_ids:
                errors.append(
                    f"Recipe '{recipe.slug}': "
                    f"required_skill_id={recipe.required_skill_id} unknown"
                )
        if recipe.required_building_type_id is not None:
            if recipe.required_building_type_id not in building_type_ids:
                errors.append(
                    f"Recipe '{recipe.slug}': "
                    f"required_building_type_id="
                    f"{recipe.required_building_type_id} unknown"
                )
        for ing in recipe.ingredients:
            if ing.item_id not in item_ids:
                errors.append(
                    f"Recipe '{recipe.slug}': "
                    f"ingredient item_id={ing.item_id} not in items"
                )
            if ing.quantity <= 0:
                errors.append(
                    f"Recipe '{recipe.slug}': "
                    f"ingredient item_id={ing.item_id} quantity={ing.quantity}"
                )

    # Buildings
    for bt in data.building_types:
        if building_cat_ids and bt.category_id not in building_cat_ids:
            errors.append(
                f"BuildingType '{bt.slug}': "
                f"category_id={bt.category_id} unknown"
            )
        if building_spec_ids and bt.specific_type_id not in building_spec_ids:
            errors.append(
                f"BuildingType '{bt.slug}': "
                f"specific_type_id={bt.specific_type_id} unknown"
            )
        for cost in bt.construction_costs:
            if cost.item_id not in item_ids:
                errors.append(
                    f"BuildingType '{bt.slug}': "
                    f"construction cost item_id={cost.item_id} not in items"
                )

    # Harvest yields
    for hy in data.harvest_yields:
        if resource_spec_ids:
            if hy.resource_specific_type_id not in resource_spec_ids:
                errors.append(
                    f"HarvestYield id={hy.id}: "
                    f"resource_specific_type_id="
                    f"{hy.resource_specific_type_id} unknown"
                )
        if hy.result_item_id not in item_ids:
            errors.append(
                f"HarvestYield id={hy.id}: "
                f"result_item_id={hy.result_item_id} not in items"
            )
        if hy.required_profession_id is not None:
            if profession_ids and hy.required_profession_id not in profession_ids:
                errors.append(
                    f"HarvestYield id={hy.id}: "
                    f"required_profession_id={hy.required_profession_id} unknown"
                )
        if hy.required_tool_item_id is not None:
            if hy.required_tool_item_id not in item_ids:
                errors.append(
                    f"HarvestYield id={hy.id}: "
                    f"required_tool_item_id={hy.required_tool_item_id} "
                    f"not in items"
                )

    # Translations (already resolved to IDs)
    entity_id_sets = {
        "item": item_ids,
        "recipe": recipe_ids,
        "building_type": building_type_ids,
        "profession": profession_ids,
        "skill": skill_ids,
    }
    for t in data.translations:
        id_set = entity_id_sets.get(t.entity_type)
        if id_set is not None and t.entity_id not in id_set:
            errors.append(
                f"Translation {t.entity_type}:{t.entity_id}.{t.field} "
                f"(lang={t.language_id}): entity_id not found"
            )

    if errors:
        logger.error("Found %d validation error(s)", len(errors))
    else:
        logger.debug("All cross-references valid")

    return errors


def _check_duplicates(
    errors: list[str], name: str, values: list[int | str]
) -> None:
    seen: set[int | str] = set()
    for val in values:
        if val in seen:
            errors.append(f"Duplicate {val!r} in {name}")
        seen.add(val)
