# Fusion

Fusion is a Rust workspace that exposes a JWT-secured HTTP API and supporting tooling for managing live-streaming metadata and user accounts. The workspace is centered on a `fusion` CLI that can boot the Axum server, run SeaORM migrations, and bundles shared crates (`api`, `entity`, `live-platform`, `migration`) to keep business logic isolated and testable.

## Highlights
- **Axum API with OpenAPI docs** – REST handlers live under `api/src/api`, expose Swagger UI at `/swagger-ui`, and use `utoipa` + `utoipa-axum` to keep routes and docs in sync.
- **Modular services** – domain logic (`api/src/service`) composes repositories (SeaORM) and helpers (Argon2 password hashing, JWT utilities) so handlers remain thin.
- **Configurable runtime** – layered `config/*.toml` files, `FUSION_*` env vars, and CLI flags (`fusion serve --port <port>`) make it easy to run in different environments.
- **Observability built-in** – request IDs, tracing subscribers, configurable logging sinks, compression, timeouts, permissive CORS, and graceful shutdown are all wired through middleware (`tower-http`).
- **Live platform providers** – the `live-platform` crate ships a `Bilibili` client and abstractions (`LivePlatform`, `LivePlatformProvider`) for querying streamer metadata and live status, ready to plug into future services.

## Repository Layout
| Path | What lives here |
| --- | --- |
| `src/` | The `fusion` CLI entrypoint (`main.rs`, `cli.rs`).
| `api/` | Axum HTTP server, routers, middleware, DTOs, config loaders, SeaORM repositories, and services.
| `entity/` | SeaORM entities generated from the schema (currently the `user` table).
| `migration/` | Database migrations and migrator CLI powered by `sea-orm-migration`.
| `live-platform/` | Provider clients (Bilibili) plus `LivePlatform` traits and DTOs.
| `config/` | Default and environment-specific TOML configuration files consumed by the API.
| `Dockerfile` | Multi-arch builder that compiles the binary statically and copies runtime config.
| `mise.toml` | Task shortcuts for formatting, checking, testing, and running via [`mise`](https://mise.jdx.dev/).

## Requirements
- Rust toolchain (nightly when using `mise`, otherwise install a compatible stable/nightly toolchain manually).
- PostgreSQL (SeaORM is configured with the `sqlx-postgres` feature; set `database.url`).
- [`mise`](https://mise.jdx.dev/) for the documented task runner (optional but recommended).
- Docker (optional) for container builds.

## Configuration
Settings are loaded in the following order, with later sources overriding earlier ones:
1. `config/default.toml`
2. `config/<environment>.toml` (where `<environment>` is `development`, `production`, or `test`)
3. `config/local.toml` (ignored by Git; create one for overrides)
4. Environment variables prefixed with `FUSION_` (nested keys use double underscores, e.g. `FUSION_DATABASE__URL`).

Key environment variables:
| Variable | Purpose |
| --- | --- |
| `FUSION_APP_ENV` | Selects the config profile (`development` is recommended locally; default is `production`).
| `FUSION_CONFIG_DIR` | Points to an alternate configuration directory (defaults to `config`).
| `FUSION_DATABASE__URL` | PostgreSQL connection string; must be provided.
| `FUSION_SERVER__PORT` | Overrides the HTTP port (`fusion serve --port` sets this automatically).
| `FUSION_JWT__SECRET` | Replace the default JWT secret in production; must be ≥32 chars.

Example `config/local.toml`:
```toml
environment = "development"

[database]
url = "postgres://fusion:fusion@localhost:5432/fusion"
run_migrations = true

[jwt]
secret = "replace_with_a_long_random_secret_for_dev"
```

## Running Locally
Using `mise`:
```bash
mise install              # installs the pinned Rust toolchain
mise run run              # formats, checks, then runs `cargo run -- serve`
```

Using plain Cargo:
```bash
cargo run -- serve
# or override the port
cargo run -- serve -- --port 9090
```
The server listens on `http://<host>:<port>` (default `0.0.0.0:8080`). Swagger UI is available at `http://localhost:8080/swagger-ui`, and the raw OpenAPI spec at `/api-docs/openapi.json`.

## Database Migrations
`src/main.rs` wires the `fusion` CLI to a `serve` subcommand (boots the API) and a `migrate` subcommand, so you can manage schema changes without touching the migration crate directly.

- Apply migrations: `fusion migrate up` (same as `cargo run -p migration -- up`).
- Roll back the last migration: `fusion migrate down` (same as `cargo run -p migration -- down`).
- The API server can also auto-run migrations on startup when `database.run_migrations = true` (default) in your config.

The first migration (`migration/src/m20251130_070455_create_user_table.rs`) creates the `user` table with unique `username` and `email` columns.

## Swagger UI
Fusion ships interactive docs via Swagger UI so you can explore routes without leaving the browser:

- Visit `http://localhost:8080/swagger-ui` once the server is running (port adjusts to whatever you pass to `fusion serve`).
- The UI is backed by an OpenAPI spec generated from the `utoipa` annotations located in `api/src/api`, so it always matches the handlers.
- Use the "Authorize" button to paste a Bearer JWT returned from `/api/v1/auth/login` or `/api/v1/auth/register`; it will be applied automatically to the protected routes.
- Download the raw spec at `http://localhost:8080/api-docs/openapi.json` for SDK generation or contract tests.

All request/response schemas are derived from the DTOs in `api/src/dto`, so updating those structs keeps the documentation in sync.

## Quality Checks & Tests
```bash
mise run fmt    # cargo fmt
mise run check  # cargo check --workspace
mise run test   # cargo test --workspace
```
Unit tests cover security-critical helpers (JWT, password hashing, auth middleware) and the Bilibili provider. When adding integration tests, place them under `api/tests/` and ensure the database schema is prepared via the migrator CLI.

## Docker
Build a single-arch image:
```bash
docker build -t fusion .
```
Or multi-arch (amd64 + arm64) via `mise run build-docker-multiarch`. The image expects configuration files under `/app/config`; mount or bake environment-specific configs and provide necessary `FUSION_*` env vars when running:
```bash
docker run --rm -p 8080:8080 \
  -e FUSION_DATABASE__URL=postgres://fusion:fusion@db:5432/fusion \
  fusion
```

## Live Platform Provider
The `live-platform` crate exposes a `LivePlatformProvider` that currently registers a Bilibili client. It can fetch streamer profiles (`StreamerInfo`) and live status (`LiveStatus`) asynchronously using `reqwest`. Services can inject this provider alongside repositories to fan out to multiple streaming platforms (Douyu, Bilibili, …) without coupling HTTP client code to handlers.

---
Need help extending Fusion? Common next steps include adding new Axum routes under `api/src/api/handlers`, enriching DTOs in `api/src/dto`, or teaching the `live-platform` provider about another platform. Run `cargo fmt && cargo check && cargo test --workspace` before opening a PR.
