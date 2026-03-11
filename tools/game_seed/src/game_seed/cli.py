"""CLI entry point for the game data seeder.

Usage:
    game-seed                     # Seed using default data/ directory and .env
    game-seed --data-dir ./data   # Custom data directory
    game-seed --dry-run           # Validate without writing to DB
    game-seed --verbose           # Detailed logging
"""

import argparse
import logging
import sys
import time
from pathlib import Path

from game_seed.loader import load_seed_data
from game_seed.seeder import Seeder
from game_seed.validators import validate_seed_data

logger = logging.getLogger(__name__)


def _setup_logging(*, verbose: bool = False) -> None:
    """Configure logging with sensible defaults."""
    level = logging.DEBUG if verbose else logging.INFO
    formatter = logging.Formatter(
        "%(asctime)s [%(levelname)s] %(name)s — %(message)s",
        datefmt="%H:%M:%S",
    )
    handler = logging.StreamHandler(sys.stdout)
    handler.setFormatter(formatter)

    root = logging.getLogger("game_seed")
    root.setLevel(level)
    root.addHandler(handler)


def _resolve_data_dir(raw: str | None) -> Path:
    """Resolve the data directory path, checking it exists."""
    if raw is not None:
        data_dir = Path(raw)
    else:
        # Default: look for data/ relative to this package, then CWD
        pkg_dir = Path(__file__).resolve().parent.parent.parent
        candidates = [
            pkg_dir / "data",
            Path.cwd() / "data",
            Path.cwd() / "tools" / "game_seed" / "data",
        ]
        data_dir = next((d for d in candidates if d.is_dir()), candidates[0])

    if not data_dir.is_dir():
        logger.error("Data directory not found: %s", data_dir)
        sys.exit(1)

    return data_dir


def _build_dsn() -> str:
    """Build PostgreSQL DSN from environment variables or .env file."""
    from dotenv import load_dotenv
    import os

    # Try .env in CWD and parent directories
    for candidate in [Path.cwd(), Path.cwd().parent, Path.cwd().parent.parent]:
        env_file = candidate / ".env"
        if env_file.exists():
            load_dotenv(env_file)
            logger.debug("Loaded .env from %s", env_file)
            break

    protocol = os.environ.get("DB_PROTOCOL", "postgresql")
    host = os.environ.get("DB_ADDRESS", "localhost")
    port = os.environ.get("DB_PORT", "5432")
    name = os.environ.get("DB_NAME", "living_landz")
    user = os.environ.get("DB_USER", "living_landz_srv")
    password = os.environ.get("DB_PASSWORD", "")

    # psycopg uses 'postgresql', not 'postgres'
    if protocol == "postgres":
        protocol = "postgresql"

    return f"{protocol}://{user}:{password}@{host}:{port}/{name}"


def main() -> None:
    """Main entry point."""
    parser = argparse.ArgumentParser(
        prog="game-seed",
        description="Declarative game data seeder for Living Landz",
    )
    parser.add_argument(
        "--data-dir",
        type=str,
        default=None,
        help="Path to the seed data directory (default: auto-detect)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Validate data without writing to the database",
    )
    parser.add_argument(
        "--verbose",
        "-v",
        action="store_true",
        help="Enable debug logging",
    )
    parser.add_argument(
        "--dsn",
        type=str,
        default=None,
        help="PostgreSQL DSN (default: built from .env)",
    )

    args = parser.parse_args()
    _setup_logging(verbose=args.verbose)

    data_dir = _resolve_data_dir(args.data_dir)
    logger.info("Data directory: %s", data_dir)

    # Load all JSON files
    seed_data = load_seed_data(data_dir)

    # Validate referential integrity
    errors = validate_seed_data(seed_data)
    if errors:
        logger.error("Validation failed with %d error(s):", len(errors))
        for err in errors:
            logger.error("  • %s", err)
        sys.exit(1)

    logger.info("Validation passed ✓")

    if args.dry_run:
        logger.info("Dry run — no changes written to database")
        sys.exit(0)

    # Connect and seed
    dsn = args.dsn or _build_dsn()
    logger.debug("Connecting to database...")

    start = time.monotonic()

    seeder = Seeder(dsn)
    try:
        report = seeder.run(seed_data)
    finally:
        seeder.close()

    elapsed = time.monotonic() - start

    # Summary
    logger.info("─" * 50)
    logger.info("Seed completed in %.1fs", elapsed)
    for table, stats in report.items():
        parts = []
        if stats.get("created"):
            parts.append(f"{stats['created']} created")
        if stats.get("updated"):
            parts.append(f"{stats['updated']} updated")
        if stats.get("archived"):
            parts.append(f"{stats['archived']} archived")
        if stats.get("unchanged"):
            parts.append(f"{stats['unchanged']} unchanged")
        if parts:
            logger.info("  %-40s %s", table, ", ".join(parts))
    logger.info("─" * 50)


if __name__ == "__main__":
    main()
