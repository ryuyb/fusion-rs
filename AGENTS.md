# Repository Guidelines

## Project Structure & Module Organization
- Root crate `fusion` (`src/main.rs`, `src/cli.rs`) hosts the CLI; the `serve` subcommand boots the API server.
- `api/` is the service surface: `api/src/api` routers and middleware, `service/` + `repository/` for business and persistence layers, `config/` and `dto/` for typed settings and payloads, `utils/` for crypto/JWT helpers.
- Supporting crates include `entity/` (SeaORM models), `migration/` (Migrator CLI + scripts), and `live-platform/` (provider clients such as `bilibili/`); shared TOML config lives under `config/`.

## Build, Test, and Development Commands
- `mise run run` (or `cargo run -- serve`) formats, checks, then runs the server.
- `mise run fmt`, `mise run check`, and `mise run test` wrap `cargo fmt`, `cargo check --workspace`, and `cargo test --workspace` for quick hygiene.
- `cargo run -p migration -- up|down|fresh` manages schema changes; append `-- status` to view applied steps.
- `docker build -t fusion .` or `mise run build-docker[-multiarch]` produces container images for deployment.

## Coding Style & Naming Conventions
- Enforce rustfmt defaults: 4-space indent, 100-col soft limit, ordered imports (`std`, third-party, workspace).
- Files and modules stay `snake_case`, public types `CamelCase`, constants `SCREAMING_SNAKE_CASE`; DTOs belong in `api/src/dto`, config structs in `api/src/config`.
- Services return `Result<T, ApiError>` and rely on the shared `tracing`; keep helpers inside `utils`.

## Testing Guidelines
- Colocate unit tests inside `#[cfg(test)] mod tests` blocks with descriptive case names (e.g., `handles_invalid_credentials`).
- Build feature-level integration tests under `api/tests/` (e.g., `user_auth.rs`) that cover at least one success and one failure path per route/service.
- Run `cargo test --workspace` before every push; DB tests must prepare schemas via the migrator CLI to avoid stale state.

## Commit & Pull Request Guidelines
- Follow Conventional Commits as seen in history (`feat: add auth handler`, `fix: wrap source anyhow error`); keep scopes concise (`feat(api): ...`).
- A PR should outline intent, link issues, and list verification steps (`cargo fmt && cargo check && cargo test --workspace`, migrations run).
- Include screenshots or `curl` transcripts whenever the HTTP contract changes, and rebase on `main` before requesting review.

## Configuration & Security Tips
- Defaults live in `config/default.toml`; override via `config/<env>.toml` or env vars consumed in `api/src/config`.
- Never commit secrets—point DB URLs, JWT keys, and provider tokens to env vars and rely on the config loader to resolve them.
- Centralize credential helpers in `api/src/utils` and audit changes touching password or token logic with another maintainer.
