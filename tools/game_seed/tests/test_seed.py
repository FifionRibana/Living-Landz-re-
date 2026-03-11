"""Tests for slug-based loader and validator."""

from pathlib import Path

from game_seed.loader import (
    RecipeDef,
    RecipeIngredient,
    load_seed_data,
)
from game_seed.validators import validate_seed_data

DATA_DIR = Path(__file__).resolve().parent.parent / "data"


class TestLoadSeedData:
    """Test that shipped data files load with slug resolution."""

    def test_loads_without_error(self) -> None:
        data = load_seed_data(DATA_DIR)
        assert len(data.items) > 0
        assert len(data.recipes) > 0
        assert len(data.building_types) > 0
        assert len(data.translations) > 0

    def test_item_slugs_are_unique(self) -> None:
        data = load_seed_data(DATA_DIR)
        slugs = [item.slug for item in data.items]
        assert len(slugs) == len(set(slugs)), "Duplicate item slugs"

    def test_recipe_slugs_are_unique(self) -> None:
        data = load_seed_data(DATA_DIR)
        slugs = [r.slug for r in data.recipes]
        assert len(slugs) == len(set(slugs)), "Duplicate recipe slugs"

    def test_slug_resolution_items(self) -> None:
        data = load_seed_data(DATA_DIR)
        # ironSword should resolve item_type to weapon (id=5)
        sword = next((i for i in data.items if i.slug == "ironSword"), None)
        assert sword is not None
        assert sword.item_type_id == 5  # weapon
        assert sword.equipment_slot_id == 6  # mainHand

    def test_slug_resolution_recipes(self) -> None:
        data = load_seed_data(DATA_DIR)
        recipe = next(
            (r for r in data.recipes if r.slug == "forgeIronSword"), None
        )
        assert recipe is not None
        assert recipe.result_item_id == 30  # ironSword
        assert recipe.required_skill_id == 5  # blacksmithing
        assert recipe.required_building_type_id == 1  # blacksmith
        # Check ingredients resolved
        iron = next(
            (i for i in recipe.ingredients if i.item_id == 6), None
        )
        assert iron is not None  # ironIngot → id 6
        assert iron.quantity == 3

    def test_slug_resolution_buildings(self) -> None:
        data = load_seed_data(DATA_DIR)
        blacksmith = next(
            (b for b in data.building_types if b.slug == "blacksmith"), None
        )
        assert blacksmith is not None
        assert blacksmith.category_id == 5  # manufacturingWorkshops
        # Construction costs resolved
        wood_cost = next(
            (c for c in blacksmith.construction_costs if c.item_id == 1), None
        )
        assert wood_cost is not None
        assert wood_cost.quantity == 10

    def test_slug_resolution_harvest(self) -> None:
        data = load_seed_data(DATA_DIR)
        wood_harvest = next(
            (h for h in data.harvest_yields if h.id == 1), None
        )
        assert wood_harvest is not None
        assert wood_harvest.resource_specific_type_id == 1  # wood
        assert wood_harvest.result_item_id == 1  # wood item
        assert wood_harvest.required_profession_id == 13  # lumberjack
        assert wood_harvest.required_tool_item_id == 21  # ironAxe

    def test_translations_resolved_to_ids(self) -> None:
        data = load_seed_data(DATA_DIR)
        wood_fr = next(
            (
                t
                for t in data.translations
                if t.entity_type == "item"
                and t.entity_id == 1  # wood → id 1
                and t.language_id == 1  # fr
                and t.field == "name"
            ),
            None,
        )
        assert wood_fr is not None
        assert wood_fr.value == "Bois"

    def test_stat_modifiers_loaded(self) -> None:
        data = load_seed_data(DATA_DIR)
        pickaxe = next((i for i in data.items if i.slug == "ironPickaxe"), None)
        assert pickaxe is not None
        assert any(m.stat_name == "mining_bonus" for m in pickaxe.stat_modifiers)


class TestValidation:
    def test_shipped_data_is_valid(self) -> None:
        data = load_seed_data(DATA_DIR)
        errors = validate_seed_data(data)
        assert errors == [], f"Validation errors: {errors}"

    def test_detects_missing_item_in_recipe(self) -> None:
        data = load_seed_data(DATA_DIR)
        data.recipes.append(
            RecipeDef(
                id=9999,
                slug="badRecipe",
                name="Bad Recipe",
                result_item_id=99999,
                ingredients=[RecipeIngredient(item_id=88888, quantity=1)],
            )
        )
        errors = validate_seed_data(data)
        assert len(errors) >= 2

    def test_detects_duplicate_slugs(self) -> None:
        data = load_seed_data(DATA_DIR)
        data.items.append(data.items[0])
        errors = validate_seed_data(data)
        assert any("Duplicate" in e for e in errors)
