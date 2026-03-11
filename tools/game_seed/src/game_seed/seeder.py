"""Declarative database seeder with slug support.

For each table:
1. Upserts all rows (INSERT ... ON CONFLICT (id) DO UPDATE), including slug
2. Archives DB rows absent from the seed (SET archived = TRUE)
3. Un-archives rows that reappear in the seed
"""

import logging
from typing import Any

import psycopg

from game_seed.loader import (
    BuildingTypeDef,
    HarvestYieldDef,
    ItemDef,
    LookupEntry,
    RecipeDef,
    SeedData,
    TranslationEntry,
)

logger = logging.getLogger(__name__)

TableStats = dict[str, int]
Report = dict[str, TableStats]


class Seeder:
    """Declarative database seeder."""

    def __init__(self, dsn: str) -> None:
        self._conn = psycopg.connect(dsn, autocommit=False)
        logger.info("Connected to database")

    def close(self) -> None:
        self._conn.close()

    def run(self, data: SeedData) -> Report:
        """Apply the full seed in a single transaction."""
        report: Report = {}
        try:
            with self._conn.cursor() as cur:
                # Lookups (order: independent first)
                report["resources.resource_categories"] = self._seed_simple_lookup(
                    cur, "resources.resource_categories", data.resource_categories
                )
                report["resources.resource_specific_types"] = (
                    self._seed_resource_specific_types(
                        cur, data.resource_specific_types
                    )
                )
                report["buildings.building_categories"] = self._seed_simple_lookup(
                    cur, "buildings.building_categories", data.building_categories
                )
                report["buildings.building_specific_types"] = (
                    self._seed_simple_lookup(
                        cur,
                        "buildings.building_specific_types",
                        data.building_specific_types,
                    )
                )
                report["resources.item_types"] = self._seed_simple_lookup(
                    cur, "resources.item_types", data.item_types
                )
                report["resources.equipment_slots"] = self._seed_simple_lookup(
                    cur, "resources.equipment_slots", data.equipment_slots
                )
                report["units.professions"] = self._seed_professions(
                    cur, data.professions
                )
                report["units.skills"] = self._seed_skills(cur, data.skills)

                # Core
                report["resources.items"] = self._seed_items(cur, data.items)
                report["buildings.building_types"] = self._seed_building_types(
                    cur, data.building_types
                )
                report["resources.recipes"] = self._seed_recipes(cur, data.recipes)

                # Relations
                report["buildings.construction_costs"] = (
                    self._seed_construction_costs(cur, data.building_types)
                )
                report["resources.harvest_yields"] = self._seed_harvest_yields(
                    cur, data.harvest_yields
                )
                report["game.translations"] = self._seed_translations(
                    cur, data.translations
                )

                self._update_sequences(cur, data)

            self._conn.commit()
            logger.info("Transaction committed")
        except Exception:
            self._conn.rollback()
            logger.exception("Seed failed — rolled back")
            raise
        return report

    # ── Simple lookup (id, slug, name, archived) ─────────────

    def _seed_simple_lookup(
        self,
        cur: psycopg.Cursor[Any],
        table: str,
        entries: list[LookupEntry],
    ) -> TableStats:
        if not entries:
            return {}
        has_archived = self._column_exists(cur, table, "archived")
        seed_ids = {e.id for e in entries}

        for e in entries:
            archived_clause = ", archived = FALSE" if has_archived else ""
            cur.execute(
                f"INSERT INTO {table} (id, slug, name) VALUES (%s, %s, %s) "
                f"ON CONFLICT (id) DO UPDATE SET "
                f"slug = EXCLUDED.slug, name = EXCLUDED.name{archived_clause}",
                (e.id, e.slug, e.name),
            )

        archived = 0
        if has_archived:
            archived = self._archive_missing(cur, table, "id", seed_ids)
        return {"upserted": len(entries), "archived": archived}

    def _seed_resource_specific_types(
        self,
        cur: psycopg.Cursor[Any],
        entries: list[LookupEntry],
    ) -> TableStats:
        if not entries:
            return {}
        seed_ids = {e.id for e in entries}
        for e in entries:
            cat_id = e.extra.get("category_id")
            cur.execute(
                "INSERT INTO resources.resource_specific_types "
                "(id, slug, name, category_id) VALUES (%s, %s, %s, %s) "
                "ON CONFLICT (id) DO UPDATE SET "
                "slug = EXCLUDED.slug, name = EXCLUDED.name, "
                "category_id = EXCLUDED.category_id, archived = FALSE",
                (e.id, e.slug, e.name, cat_id),
            )
        archived = self._archive_missing(
            cur, "resources.resource_specific_types", "id", seed_ids
        )
        return {"upserted": len(entries), "archived": archived}

    def _seed_professions(
        self, cur: psycopg.Cursor[Any], entries: list[LookupEntry]
    ) -> TableStats:
        if not entries:
            return {}
        seed_ids = {e.id for e in entries}
        for e in entries:
            cur.execute(
                "INSERT INTO units.professions "
                "(id, slug, name, description, base_inventory_capacity_bonus) "
                "VALUES (%s, %s, %s, %s, %s) "
                "ON CONFLICT (id) DO UPDATE SET "
                "slug = EXCLUDED.slug, name = EXCLUDED.name, "
                "description = EXCLUDED.description, "
                "base_inventory_capacity_bonus = "
                "EXCLUDED.base_inventory_capacity_bonus, "
                "archived = FALSE",
                (
                    e.id,
                    e.slug,
                    e.name,
                    e.extra.get("description", ""),
                    e.extra.get("base_inventory_capacity_bonus", 0),
                ),
            )
        archived = self._archive_missing(
            cur, "units.professions", "id", seed_ids
        )
        return {"upserted": len(entries), "archived": archived}

    def _seed_skills(
        self, cur: psycopg.Cursor[Any], entries: list[LookupEntry]
    ) -> TableStats:
        if not entries:
            return {}
        seed_ids = {e.id for e in entries}
        for e in entries:
            cur.execute(
                "INSERT INTO units.skills "
                "(id, slug, name, description, primary_stat) "
                "VALUES (%s, %s, %s, %s, %s) "
                "ON CONFLICT (id) DO UPDATE SET "
                "slug = EXCLUDED.slug, name = EXCLUDED.name, "
                "description = EXCLUDED.description, "
                "primary_stat = EXCLUDED.primary_stat, "
                "archived = FALSE",
                (
                    e.id,
                    e.slug,
                    e.name,
                    e.extra.get("description", ""),
                    e.extra.get("primary_stat", "strength"),
                ),
            )
        archived = self._archive_missing(cur, "units.skills", "id", seed_ids)
        return {"upserted": len(entries), "archived": archived}

    # ── Items ────────────────────────────────────────────────

    def _seed_items(
        self, cur: psycopg.Cursor[Any], items: list[ItemDef]
    ) -> TableStats:
        if not items:
            return {}
        seed_ids = {i.id for i in items}
        for item in items:
            cur.execute(
                "INSERT INTO resources.items "
                "(id, slug, name, item_type_id, category_id, "
                "weight_kg, volume_liters, base_price, "
                "is_perishable, base_decay_rate_per_day, "
                "is_equipable, equipment_slot_id, "
                "is_craftable, description, archived) "
                "VALUES (%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,FALSE) "
                "ON CONFLICT (id) DO UPDATE SET "
                "slug=EXCLUDED.slug, name=EXCLUDED.name, "
                "item_type_id=EXCLUDED.item_type_id, "
                "category_id=EXCLUDED.category_id, "
                "weight_kg=EXCLUDED.weight_kg, "
                "volume_liters=EXCLUDED.volume_liters, "
                "base_price=EXCLUDED.base_price, "
                "is_perishable=EXCLUDED.is_perishable, "
                "base_decay_rate_per_day=EXCLUDED.base_decay_rate_per_day, "
                "is_equipable=EXCLUDED.is_equipable, "
                "equipment_slot_id=EXCLUDED.equipment_slot_id, "
                "is_craftable=EXCLUDED.is_craftable, "
                "description=EXCLUDED.description, archived=FALSE",
                (
                    item.id, item.slug, item.name, item.item_type_id,
                    item.category_id, item.weight_kg, item.volume_liters,
                    item.base_price, item.is_perishable,
                    item.base_decay_rate_per_day, item.is_equipable,
                    item.equipment_slot_id, item.is_craftable, item.description,
                ),
            )
            # Replace stat modifiers
            cur.execute(
                "DELETE FROM resources.item_stat_modifiers WHERE item_id = %s",
                (item.id,),
            )
            for mod in item.stat_modifiers:
                cur.execute(
                    "INSERT INTO resources.item_stat_modifiers "
                    "(item_id, stat_name, modifier_value) VALUES (%s,%s,%s)",
                    (item.id, mod.stat_name, mod.modifier_value),
                )
        archived = self._archive_missing(
            cur, "resources.items", "id", seed_ids
        )
        return {"upserted": len(items), "archived": archived}

    # ── Recipes ──────────────────────────────────────────────

    def _seed_recipes(
        self, cur: psycopg.Cursor[Any], recipes: list[RecipeDef]
    ) -> TableStats:
        if not recipes:
            return {}
        seed_ids = {r.id for r in recipes}
        for recipe in recipes:
            cur.execute(
                "INSERT INTO resources.recipes "
                "(id, slug, name, description, result_item_id, "
                "result_quantity, required_skill_id, required_skill_level, "
                "craft_duration_seconds, required_building_type_id, archived) "
                "VALUES (%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,FALSE) "
                "ON CONFLICT (id) DO UPDATE SET "
                "slug=EXCLUDED.slug, name=EXCLUDED.name, "
                "description=EXCLUDED.description, "
                "result_item_id=EXCLUDED.result_item_id, "
                "result_quantity=EXCLUDED.result_quantity, "
                "required_skill_id=EXCLUDED.required_skill_id, "
                "required_skill_level=EXCLUDED.required_skill_level, "
                "craft_duration_seconds=EXCLUDED.craft_duration_seconds, "
                "required_building_type_id="
                "EXCLUDED.required_building_type_id, archived=FALSE",
                (
                    recipe.id, recipe.slug, recipe.name, recipe.description,
                    recipe.result_item_id, recipe.result_quantity,
                    recipe.required_skill_id, recipe.required_skill_level,
                    recipe.craft_duration_seconds,
                    recipe.required_building_type_id,
                ),
            )
            cur.execute(
                "DELETE FROM resources.recipe_ingredients "
                "WHERE recipe_id = %s",
                (recipe.id,),
            )
            for ing in recipe.ingredients:
                cur.execute(
                    "INSERT INTO resources.recipe_ingredients "
                    "(recipe_id, item_id, quantity) VALUES (%s,%s,%s)",
                    (recipe.id, ing.item_id, ing.quantity),
                )
        archived = self._archive_missing(
            cur, "resources.recipes", "id", seed_ids
        )
        return {"upserted": len(recipes), "archived": archived}

    # ── Building types ───────────────────────────────────────

    def _seed_building_types(
        self, cur: psycopg.Cursor[Any], buildings: list[BuildingTypeDef]
    ) -> TableStats:
        if not buildings:
            return {}
        seed_ids = {bt.id for bt in buildings}
        for bt in buildings:
            cur.execute(
                "INSERT INTO buildings.building_types "
                "(id, slug, name, category_id, specific_type_id, "
                "description, archived) "
                "VALUES (%s,%s,%s,%s,%s,%s,FALSE) "
                "ON CONFLICT (id) DO UPDATE SET "
                "slug=EXCLUDED.slug, name=EXCLUDED.name, "
                "category_id=EXCLUDED.category_id, "
                "specific_type_id=EXCLUDED.specific_type_id, "
                "description=EXCLUDED.description, archived=FALSE",
                (
                    bt.id, bt.slug, bt.name, bt.category_id,
                    bt.specific_type_id, bt.description,
                ),
            )
        archived = self._archive_missing(
            cur, "buildings.building_types", "id", seed_ids
        )
        return {"upserted": len(buildings), "archived": archived}

    # ── Construction costs ───────────────────────────────────

    def _seed_construction_costs(
        self, cur: psycopg.Cursor[Any], buildings: list[BuildingTypeDef]
    ) -> TableStats:
        if not buildings:
            return {}
        bt_ids = {bt.id for bt in buildings}
        total = 0
        for bt in buildings:
            cur.execute(
                "DELETE FROM buildings.construction_costs "
                "WHERE building_type_id = %s",
                (bt.id,),
            )
            for cost in bt.construction_costs:
                cur.execute(
                    "INSERT INTO buildings.construction_costs "
                    "(building_type_id, item_id, quantity) VALUES (%s,%s,%s)",
                    (bt.id, cost.item_id, cost.quantity),
                )
                total += 1
        # Remove orphan costs
        if bt_ids:
            ph = ",".join(["%s"] * len(bt_ids))
            cur.execute(
                f"DELETE FROM buildings.construction_costs "
                f"WHERE building_type_id NOT IN ({ph})",
                list(bt_ids),
            )
        return {"upserted": total}

    # ── Harvest yields ───────────────────────────────────────

    def _seed_harvest_yields(
        self, cur: psycopg.Cursor[Any], yields: list[HarvestYieldDef]
    ) -> TableStats:
        if not yields:
            return {}
        seed_ids = {hy.id for hy in yields}
        for hy in yields:
            cur.execute(
                "INSERT INTO resources.harvest_yields "
                "(id, resource_specific_type_id, result_item_id, "
                "base_quantity, quality_min, quality_max, "
                "required_profession_id, required_tool_item_id, "
                "tool_bonus_quantity, duration_seconds) "
                "VALUES (%s,%s,%s,%s,%s,%s,%s,%s,%s,%s) "
                "ON CONFLICT (id) DO UPDATE SET "
                "resource_specific_type_id="
                "EXCLUDED.resource_specific_type_id, "
                "result_item_id=EXCLUDED.result_item_id, "
                "base_quantity=EXCLUDED.base_quantity, "
                "quality_min=EXCLUDED.quality_min, "
                "quality_max=EXCLUDED.quality_max, "
                "required_profession_id=EXCLUDED.required_profession_id, "
                "required_tool_item_id=EXCLUDED.required_tool_item_id, "
                "tool_bonus_quantity=EXCLUDED.tool_bonus_quantity, "
                "duration_seconds=EXCLUDED.duration_seconds",
                (
                    hy.id, hy.resource_specific_type_id, hy.result_item_id,
                    hy.base_quantity, hy.quality_min, hy.quality_max,
                    hy.required_profession_id, hy.required_tool_item_id,
                    hy.tool_bonus_quantity, hy.duration_seconds,
                ),
            )
        if seed_ids:
            ph = ",".join(["%s"] * len(seed_ids))
            cur.execute(
                f"DELETE FROM resources.harvest_yields "
                f"WHERE id NOT IN ({ph})",
                list(seed_ids),
            )
        return {"upserted": len(yields)}

    # ── Translations ─────────────────────────────────────────

    def _seed_translations(
        self, cur: psycopg.Cursor[Any], translations: list[TranslationEntry]
    ) -> TableStats:
        if not translations:
            return {}
        seed_keys: set[tuple[str, int, int, str]] = set()
        for t in translations:
            seed_keys.add((t.entity_type, t.entity_id, t.language_id, t.field))
            cur.execute(
                "INSERT INTO game.translations "
                "(entity_type, entity_id, language_id, field, value) "
                "VALUES (%s,%s,%s,%s,%s) "
                "ON CONFLICT (entity_type, entity_id, language_id, field) "
                "DO UPDATE SET value = EXCLUDED.value",
                (t.entity_type, t.entity_id, t.language_id, t.field, t.value),
            )
        # Remove orphan translations
        cur.execute(
            "SELECT entity_type, entity_id, language_id, field "
            "FROM game.translations"
        )
        existing = {
            (row[0], row[1], row[2], row[3]) for row in cur.fetchall()
        }
        orphans = existing - seed_keys
        for et, eid, lid, fld in orphans:
            cur.execute(
                "DELETE FROM game.translations "
                "WHERE entity_type=%s AND entity_id=%s "
                "AND language_id=%s AND field=%s",
                (et, eid, lid, fld),
            )
        if orphans:
            logger.info("  Removed %d orphaned translation(s)", len(orphans))
        return {"upserted": len(translations), "archived": len(orphans)}

    # ── Helpers ───────────────────────────────────────────────

    def _archive_missing(
        self, cur: psycopg.Cursor[Any], table: str,
        id_col: str, seed_ids: set[int],
    ) -> int:
        if not seed_ids:
            return 0
        ph = ",".join(["%s"] * len(seed_ids))
        cur.execute(
            f"UPDATE {table} SET archived = TRUE "
            f"WHERE {id_col} NOT IN ({ph}) AND archived = FALSE",
            list(seed_ids),
        )
        count = cur.rowcount
        if count:
            logger.info(
                "  Archived %d row(s) from %s not in seed", count, table
            )
        return count

    def _column_exists(
        self, cur: psycopg.Cursor[Any], table: str, column: str
    ) -> bool:
        parts = table.split(".")
        schema = parts[0] if len(parts) > 1 else "public"
        table_name = parts[-1]
        cur.execute(
            "SELECT 1 FROM information_schema.columns "
            "WHERE table_schema=%s AND table_name=%s AND column_name=%s",
            (schema, table_name, column),
        )
        return cur.fetchone() is not None

    def _update_sequences(
        self, cur: psycopg.Cursor[Any], data: SeedData
    ) -> None:
        seqs = [
            ("resources.items", "resources.items_id_seq", data.items),
            ("resources.recipes", "resources.recipes_id_seq", data.recipes),
            (
                "buildings.building_types",
                "buildings.building_types_id_seq",
                data.building_types,
            ),
            (
                "resources.harvest_yields",
                "resources.harvest_yields_id_seq",
                data.harvest_yields,
            ),
        ]
        for table, seq, entries in seqs:
            if not entries:
                continue
            max_id = max(e.id for e in entries)
            try:
                cur.execute(
                    f"SELECT setval('{seq}', GREATEST({max_id}, "
                    f"(SELECT COALESCE(MAX(id), 0) FROM {table})))"
                )
            except Exception:
                logger.debug("Could not update sequence %s", seq, exc_info=True)
