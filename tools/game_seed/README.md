# game-seed

Declarative game data seeder for Living Landz.

The JSON files in `data/` are the **single source of truth** for all game definitions (items, recipes, buildings, translations). The seeder applies them to PostgreSQL at every server startup, ensuring the database always matches the versioned data.

## How it works

1. Loads all JSON files from `data/`
2. Validates referential integrity in-memory (all foreign keys, no duplicate IDs)
3. Upserts every row (`INSERT ... ON CONFLICT DO UPDATE`)
4. **Archives** any DB row whose ID is absent from the seed (`SET archived = TRUE`)
5. Reports what was created, updated, or archived

## Setup

```bash
cd tools/game_seed
uv sync
```

## Usage

```bash
# Full seed (reads .env for DB connection)
uv run game-seed

# Validate without touching the database
uv run game-seed --dry-run

# Verbose output
uv run game-seed -v

# Custom data directory
uv run game-seed --data-dir /path/to/data

# Explicit DSN
uv run game-seed --dsn postgresql://user:pass@localhost:5432/living_landz
```

## Prerequisite migration

Before the first seed, run the migration that creates `game.translations`, `buildings.construction_costs`, and `resources.harvest_yields`:

```bash
psql -f migration_seed_tables.sql $DATABASE_URL
```

Or copy the file into `migrations/` with a proper timestamp prefix.

## Data files

| File | Tables seeded |
|---|---|
| `lookups.json` | resource_categories, resource_specific_types, building_categories, building_specific_types, item_types, equipment_slots, professions, skills |
| `items.json` | resources.items, resources.item_stat_modifiers |
| `recipes.json` | resources.recipes, resources.recipe_ingredients |
| `buildings.json` | buildings.building_types, buildings.construction_costs |
| `harvest.json` | resources.harvest_yields |
| `translations.json` | game.translations |

All files are optional — if missing, that domain is skipped.

## Slugs

Every entity has a `slug` — a camelCase identifier used as the cross-reference key in JSON files. Slugs are resolved to numeric IDs by the loader at parse time and stored in the DB alongside the numeric ID.

```json
// recipes.json — human-readable references
{"slug": "forgeIronSword", "result_item": "ironSword",
 "required_building": "blacksmith", "required_skill": "blacksmithing",
 "ingredients": [{"item": "ironIngot", "quantity": 3}, {"item": "wood", "quantity": 1}]}
```

The loader resolves `"ironSword"` → `30`, `"blacksmith"` → `1`, etc. before anything reaches the DB. If a slug is unknown, loading fails immediately with a clear error message.

The DB stores both: `id=30, slug='ironSword'`. Rust code continues to work with numeric IDs. The slugs are there for traceability and future tooling.

## Translations format

Translations use slugs as keys for readability:

```json
{
  "item": {
    "ironSword": {
      "name": {"fr": "Épée en fer", "en": "Iron Sword", "de": "Eisenschwert"}
    }
  }
}
```

To add a new language: insert the language in `game.languages`, add the code to `LANG_CODES` in `loader.py`, and add translations in the JSON.

## Archival behavior

The seeder is **declarative**: the JSON files define the complete set of active data. If you remove an item from `items.json`, it will be marked `archived = TRUE` in the database on the next seed run. It is never deleted, so existing item instances (player inventories) remain valid.

The seeder logs every archival:
```
Archived 2 row(s) from resources.items not present in seed
```

To restore an archived entity, simply add it back to the JSON file.

## Rust integration

Call the seeder from the Rust server before initializing caches:

```rust
let status = std::process::Command::new("uv")
    .args(["run", "game-seed"])
    .current_dir("tools/game_seed")
    .status()
    .expect("Failed to run game-seed");

if !status.success() {
    panic!("Game seed failed");
}
```
