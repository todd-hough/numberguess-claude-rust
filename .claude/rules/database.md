---
paths:
  - "src/db/**"
  - "migrations/**"
  - "src/main.rs"
---
# Database (PostgreSQL via SQLx)

- Repository pattern: `GameRepository` trait (`src/db/repository.rs`, native async fn in
  traits) implemented by `PostgresGameRepository` (`src/db/postgres_repository.rs`);
  `DbError` lives in `src/db/mod.rs`. Handlers are generic over `R: GameRepository`
  (static dispatch).
- Queries are runtime-checked (no `query!` macros), so `cargo build` needs no database —
  keep it that way.
- Schema: one `games` row per active game — `game_id` (BIGINT PK), `min_value`,
  `max_value`, `secret_number`, `guess_count`, `max_guesses` (nullable), timestamps.
  Rows are deleted when a game completes.
- A guess runs in a single transaction using `SELECT ... FOR UPDATE`.
- Migrations in `migrations/` run automatically at server startup.
  `cleanup_old_games(hours_old DEFAULT 24)` removes abandoned games; nothing schedules it.
- Pool: SQLx `PgPool`, default 5 connections; tune with `DB_MAX_CONNECTIONS` (clamped to
  1–100; unparseable values fall back to 5 with a warning).
- `DATABASE_URL` is required in server mode (env or `.env`; no `.env` is checked in).
  Defaults in compose and the Makefile: user `numberguess`, password `password`, database
  `numberguess_dev` → `postgresql://numberguess:password@localhost:5432/numberguess_dev`.
  Override with `POSTGRES_USER` / `POSTGRES_PASSWORD` / `POSTGRES_DB`
  (e.g. `POSTGRES_DB=mydb make dev`). The full test tier uses `numberguess_test`.
- `make db-shell` opens psql.
