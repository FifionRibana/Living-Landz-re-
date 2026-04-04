# TODO Audit Report — Issue #131

Audit date: 2026-04-05
Audited by: automated (Claude Code)
Total TODOs found: 39
Deleted (obsolete): 4
Updated with issue reference: 1
Standardized/translated: 34

## Summary by area

| Area | Count | Notes |
|------|-------|-------|
| Client UI | 15 | Mostly minor UX improvements |
| Client networking | 0 | Cleaned up (was 1 obsolete) |
| Client grid/camera | 3 | Movement logic, config placement |
| Server action processor | 1 | Missing fields |
| Server database | 6 | Road lookup hardcoding, action loading |
| Server networking | 3 | Room-based targeting (not TODO, now NOTE) |
| Server world | 4 | Utils extraction, tests |
| Shared types | 4 | DB migration, i18n |
| Shared protocol | 1 | Netcode key (#194) |

## TODOs linked to existing issues

- `netcode_config.rs` → #194 (Security: secrets in repo)

## TODOs that may warrant new issues

- Road lookup hardcoding (4 occurrences) — consider a single issue for road type DB migration
- Building enum data to database — relates to future P1-06 Construction & Urbanism milestone
- Profession training durations to database — relates to future P1-04 Stats & Skills milestone
