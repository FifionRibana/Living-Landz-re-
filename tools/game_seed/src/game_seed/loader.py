"""Load seed data from JSON files and resolve slug references to numeric IDs.

The JSON files use human-readable slugs (e.g. "ironSword", "blacksmith")
for all cross-references. This module resolves them to numeric IDs
before the data reaches the seeder or validator.

Loading order matters — lookups first, then items, then recipes/buildings
that reference items.
"""

import json
import logging
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

logger = logging.getLogger(__name__)

LANG_CODES: dict[str, int] = {
    "fr": 1,
    "en": 2,
    "de": 3,
    "es": 4,
    "it": 5,
}


# ── Slug resolver ────────────────────────────────────────────


class SlugResolver:
    """Builds and queries slug → numeric ID maps by namespace.

    Each namespace (e.g. "item", "building_type", "skill") maintains
    its own independent slug → id mapping.
    """

    def __init__(self) -> None:
        self._maps: dict[str, dict[str, int]] = {}

    def register(self, namespace: str, slug: str, numeric_id: int) -> None:
        """Register a slug → id mapping in the given namespace."""
        ns = self._maps.setdefault(namespace, {})
        if slug in ns and ns[slug] != numeric_id:
            logger.warning(
                "Slug '%s' re-registered in namespace '%s': %d → %d",
                slug,
                namespace,
                ns[slug],
                numeric_id,
            )
        ns[slug] = numeric_id

    def resolve(self, namespace: str, slug: str | None) -> int | None:
        """Resolve a slug to its numeric ID. Returns None if slug is None."""
        if slug is None:
            return None
        ns = self._maps.get(namespace, {})
        if slug not in ns:
            raise KeyError(
                f"Unknown slug '{slug}' in namespace '{namespace}'. "
                f"Known: {sorted(ns.keys())}"
            )
        return ns[slug]

    def resolve_required(self, namespace: str, slug: str) -> int:
        """Like resolve(), but raises if slug is None or missing."""
        result = self.resolve(namespace, slug)
        if result is None:
            raise KeyError(f"Required slug is None in namespace '{namespace}'")
        return result

    def has(self, namespace: str, slug: str) -> bool:
        """Check if a slug is registered."""
        return slug in self._maps.get(namespace, {})

    def all_slugs(self, namespace: str) -> set[str]:
        """Return all registered slugs for a namespace."""
        return set(self._maps.get(namespace, {}).keys())


# ── Data models ──────────────────────────────────────────────


@dataclass
class ItemStatModifier:
    stat_name: str
    modifier_value: int


@dataclass
class ItemDef:
    id: int
    slug: str
    name: str
    item_type_id: int
    category_id: int | None = None
    weight_kg: float = 0.001
    volume_liters: float = 0.001
    base_price: int = 0
    is_perishable: bool = False
    base_decay_rate_per_day: float = 0.0
    is_equipable: bool = False
    equipment_slot_id: int | None = None
    is_craftable: bool = False
    description: str = ""
    stat_modifiers: list[ItemStatModifier] = field(default_factory=list)


@dataclass
class RecipeIngredient:
    item_id: int
    quantity: int


@dataclass
class RecipeDef:
    id: int
    slug: str
    name: str
    description: str = ""
    result_item_id: int = 0
    result_quantity: int = 1
    required_skill_id: int | None = None
    required_skill_level: int = 1
    craft_duration_seconds: int = 10
    required_building_type_id: int | None = None
    ingredients: list[RecipeIngredient] = field(default_factory=list)


@dataclass
class ConstructionCost:
    item_id: int
    quantity: int


@dataclass
class BuildingTypeDef:
    id: int
    slug: str
    name: str
    category_id: int
    specific_type_id: int
    description: str = ""
    construction_duration_seconds: int = 15
    construction_costs: list[ConstructionCost] = field(default_factory=list)


@dataclass
class HarvestYieldDef:
    id: int
    resource_specific_type_id: int
    result_item_id: int
    base_quantity: int = 1
    quality_min: float = 0.5
    quality_max: float = 1.0
    required_profession_id: int | None = None
    required_tool_item_id: int | None = None
    tool_bonus_quantity: int = 0
    duration_seconds: int = 30


@dataclass
class LookupEntry:
    id: int
    slug: str
    name: str
    extra: dict[str, Any] = field(default_factory=dict)


@dataclass
class TranslationEntry:
    entity_type: str
    entity_id: int
    language_id: int
    field: str
    value: str


@dataclass
class SeedData:
    resource_categories: list[LookupEntry] = field(default_factory=list)
    resource_specific_types: list[LookupEntry] = field(default_factory=list)
    building_categories: list[LookupEntry] = field(default_factory=list)
    building_specific_types: list[LookupEntry] = field(default_factory=list)
    item_types: list[LookupEntry] = field(default_factory=list)
    equipment_slots: list[LookupEntry] = field(default_factory=list)
    professions: list[LookupEntry] = field(default_factory=list)
    skills: list[LookupEntry] = field(default_factory=list)

    items: list[ItemDef] = field(default_factory=list)
    recipes: list[RecipeDef] = field(default_factory=list)
    building_types: list[BuildingTypeDef] = field(default_factory=list)
    harvest_yields: list[HarvestYieldDef] = field(default_factory=list)

    translations: list[TranslationEntry] = field(default_factory=list)


# ── JSON loading ─────────────────────────────────────────────


def _load_json(path: Path) -> dict[str, Any]:
    if not path.exists():
        logger.debug("Skipping missing file: %s", path.name)
        return {}
    with path.open(encoding="utf-8") as f:
        data = json.load(f)
    logger.info("Loaded %s", path.name)
    return data


# ── Lookup parsing ───────────────────────────────────────────


def _parse_and_register_lookups(
    raw: dict[str, Any],
    key: str,
    namespace: str,
    resolver: SlugResolver,
) -> list[LookupEntry]:
    entries = []
    for item in raw.get(key, []):
        slug = item["slug"]
        entry = LookupEntry(
            id=item["id"],
            slug=slug,
            name=item["name"],
            extra={
                k: v for k, v in item.items() if k not in ("id", "slug", "name")
            },
        )
        resolver.register(namespace, slug, entry.id)
        entries.append(entry)
    return entries


# ── Item parsing ─────────────────────────────────────────────


def _parse_items(
    raw: dict[str, Any], resolver: SlugResolver
) -> list[ItemDef]:
    items = []
    for item in raw.get("items", []):
        slug = item["slug"]
        modifiers = [
            ItemStatModifier(stat_name=k, modifier_value=v)
            for k, v in item.get("stat_modifiers", {}).items()
        ]

        # Resolve slug references
        item_type_id = resolver.resolve_required("item_type", item["item_type"])
        category_id = resolver.resolve("resource_category", item.get("category"))
        equipment_slot_id = resolver.resolve(
            "equipment_slot", item.get("equipment_slot")
        )

        parsed = ItemDef(
            id=item["id"],
            slug=slug,
            name=item["name"],
            item_type_id=item_type_id,
            category_id=category_id,
            weight_kg=item.get("weight_kg", 0.001),
            volume_liters=item.get("volume_liters", 0.001),
            base_price=item.get("base_price", 0),
            is_perishable=item.get("is_perishable", False),
            base_decay_rate_per_day=item.get("base_decay_rate_per_day", 0.0),
            is_equipable=item.get("is_equipable", False),
            equipment_slot_id=equipment_slot_id,
            is_craftable=item.get("is_craftable", False),
            description=item.get("description", ""),
            stat_modifiers=modifiers,
        )
        resolver.register("item", slug, parsed.id)
        items.append(parsed)
    return items


# ── Recipe parsing ───────────────────────────────────────────


def _parse_recipes(
    raw: dict[str, Any], resolver: SlugResolver
) -> list[RecipeDef]:
    recipes = []
    for recipe in raw.get("recipes", []):
        slug = recipe["slug"]
        ingredients = [
            RecipeIngredient(
                item_id=resolver.resolve_required("item", ing["item"]),
                quantity=ing["quantity"],
            )
            for ing in recipe.get("ingredients", [])
        ]

        parsed = RecipeDef(
            id=recipe["id"],
            slug=slug,
            name=recipe["name"],
            description=recipe.get("description", ""),
            result_item_id=resolver.resolve_required(
                "item", recipe["result_item"]
            ),
            result_quantity=recipe.get("result_quantity", 1),
            required_skill_id=resolver.resolve(
                "skill", recipe.get("required_skill")
            ),
            required_skill_level=recipe.get("required_skill_level", 1),
            craft_duration_seconds=recipe.get("craft_duration_seconds", 10),
            required_building_type_id=resolver.resolve(
                "building_type", recipe.get("required_building")
            ),
            ingredients=ingredients,
        )
        resolver.register("recipe", slug, parsed.id)
        recipes.append(parsed)
    return recipes


# ── Building parsing ─────────────────────────────────────────


def _parse_buildings(
    raw: dict[str, Any], resolver: SlugResolver
) -> list[BuildingTypeDef]:
    buildings = []
    for bt in raw.get("building_types", []):
        slug = bt["slug"]
        costs = [
            ConstructionCost(
                item_id=resolver.resolve_required("item", c["item"]),
                quantity=c["quantity"],
            )
            for c in bt.get("construction_costs", [])
        ]

        parsed = BuildingTypeDef(
            id=bt["id"],
            slug=slug,
            name=bt["name"],
            category_id=resolver.resolve_required(
                "building_category", bt["category"]
            ),
            specific_type_id=resolver.resolve_required(
                "building_specific_type", bt["specific_type"]
            ),
            description=bt.get("description", ""),
            construction_duration_seconds=bt.get("construction_duration_seconds", 15),
            construction_costs=costs,
        )
        resolver.register("building_type", slug, parsed.id)
        buildings.append(parsed)
    return buildings


# ── Harvest parsing ──────────────────────────────────────────


def _parse_harvest(
    raw: dict[str, Any], resolver: SlugResolver
) -> list[HarvestYieldDef]:
    yields = []
    for hy in raw.get("harvest_yields", []):
        yields.append(
            HarvestYieldDef(
                id=hy["id"],
                resource_specific_type_id=resolver.resolve_required(
                    "resource_specific_type", hy["resource_type"]
                ),
                result_item_id=resolver.resolve_required(
                    "item", hy["result_item"]
                ),
                base_quantity=hy.get("base_quantity", 1),
                quality_min=hy.get("quality_min", 0.5),
                quality_max=hy.get("quality_max", 1.0),
                required_profession_id=resolver.resolve(
                    "profession", hy.get("required_profession")
                ),
                required_tool_item_id=resolver.resolve(
                    "item", hy.get("required_tool")
                ),
                tool_bonus_quantity=hy.get("tool_bonus_quantity", 0),
                duration_seconds=hy.get("duration_seconds", 30),
            )
        )
    return yields


# ── Translation parsing ──────────────────────────────────────


def _parse_translations(
    raw: dict[str, Any], resolver: SlugResolver
) -> list[TranslationEntry]:
    """Parse translations, resolving slug keys to numeric entity IDs.

    Expected format::

        {
            "item": {
                "ironSword": {"name": {"fr": "Épée en fer", "en": "Iron Sword"}}
            }
        }
    """
    # Map translation entity_type → resolver namespace
    namespace_map = {
        "item": "item",
        "recipe": "recipe",
        "building_type": "building_type",
        "profession": "profession",
        "skill": "skill",
    }

    entries = []
    for entity_type, entities in raw.items():
        namespace = namespace_map.get(entity_type)
        if namespace is None:
            logger.warning("Unknown translation entity_type '%s'", entity_type)
            continue

        for slug, fields in entities.items():
            entity_id = resolver.resolve(namespace, slug)
            if entity_id is None:
                logger.warning(
                    "Translation references unknown slug '%s' "
                    "in namespace '%s' — skipping",
                    slug,
                    namespace,
                )
                continue

            for field_name, lang_values in fields.items():
                for lang_code, value in lang_values.items():
                    language_id = LANG_CODES.get(lang_code)
                    if language_id is None:
                        logger.warning(
                            "Unknown language code '%s' for %s:%s.%s",
                            lang_code,
                            entity_type,
                            slug,
                            field_name,
                        )
                        continue
                    entries.append(
                        TranslationEntry(
                            entity_type=entity_type,
                            entity_id=entity_id,
                            language_id=language_id,
                            field=field_name,
                            value=value,
                        )
                    )
    return entries


# ── Main loader ──────────────────────────────────────────────


def load_seed_data(data_dir: Path) -> SeedData:
    """Load all seed data, resolving slug references to numeric IDs.

    Loading order:
      1. lookups (build slug→id maps)
      2. items (reference lookup slugs, register item slugs)
      3. buildings (reference lookup + item slugs, register building slugs)
      4. recipes (reference item + skill + building slugs)
      5. harvest (reference resource_specific_type + item + profession slugs)
      6. translations (resolve all slugs to IDs)
    """
    resolver = SlugResolver()

    # 1. Lookups
    lookups_raw = _load_json(data_dir / "lookups.json")

    resource_categories = _parse_and_register_lookups(
        lookups_raw, "resource_categories", "resource_category", resolver
    )
    resource_specific_types = _parse_and_register_lookups(
        lookups_raw, "resource_specific_types", "resource_specific_type", resolver
    )
    building_categories = _parse_and_register_lookups(
        lookups_raw, "building_categories", "building_category", resolver
    )
    building_specific_types = _parse_and_register_lookups(
        lookups_raw, "building_specific_types", "building_specific_type", resolver
    )
    item_types = _parse_and_register_lookups(
        lookups_raw, "item_types", "item_type", resolver
    )
    equipment_slots = _parse_and_register_lookups(
        lookups_raw, "equipment_slots", "equipment_slot", resolver
    )
    professions = _parse_and_register_lookups(
        lookups_raw, "professions", "profession", resolver
    )
    skills = _parse_and_register_lookups(
        lookups_raw, "skills", "skill", resolver
    )

    # Resolve category slug in resource_specific_types
    for entry in resource_specific_types:
        cat_slug = entry.extra.pop("category", None)
        if cat_slug is not None:
            entry.extra["category_id"] = resolver.resolve_required(
                "resource_category", cat_slug
            )

    # 2. Items
    items_raw = _load_json(data_dir / "items.json")
    items = _parse_items(items_raw, resolver)

    # 3. Buildings (need item slugs for construction costs)
    buildings_raw = _load_json(data_dir / "buildings.json")
    building_types = _parse_buildings(buildings_raw, resolver)

    # 4. Recipes (need item + skill + building slugs)
    recipes_raw = _load_json(data_dir / "recipes.json")
    recipes = _parse_recipes(recipes_raw, resolver)

    # 5. Harvest (need resource_specific_type + item + profession slugs)
    harvest_raw = _load_json(data_dir / "harvest.json")
    harvest_yields = _parse_harvest(harvest_raw, resolver)

    # 6. Translations (need all slugs)
    translations_raw = _load_json(data_dir / "translations.json")
    translations = _parse_translations(translations_raw, resolver)

    return SeedData(
        resource_categories=resource_categories,
        resource_specific_types=resource_specific_types,
        building_categories=building_categories,
        building_specific_types=building_specific_types,
        item_types=item_types,
        equipment_slots=equipment_slots,
        professions=professions,
        skills=skills,
        items=items,
        recipes=recipes,
        building_types=building_types,
        harvest_yields=harvest_yields,
        translations=translations,
    )
