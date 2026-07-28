# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```sh
cargo build --release
cargo run -- --workdir ./data serve --port 5678   # run the server locally
cargo check                                        # fast type-check loop
cargo clippy --all-targets
cargo fmt

docker compose up --build                          # containerized run, port 5678
```

There are currently **no tests** in this repo (no `tests/` directory, no `#[cfg(test)]` modules). `cargo test` succeeds vacuously. If adding tests, `cargo test <name>` runs a single one.

`WORKDIR` and `PORT` env vars back the `--workdir` / `--port` flags (clap `env` attribute), and `main.rs` loads `.env` via `dotenv`. Verbosity comes from `clap-verbosity-flag` (`-v`, `-vv`, …), defaulting to info.

CI: `.github/workflows/docker-image.yml` builds and pushes the Docker image; `publish-furdb-crate.yml` publishes to crates.io and is `workflow_dispatch`-only.

## Architecture

Single binary crate with a strict two-layer split:

- **`src/core/`** — the storage engine. Pure filesystem + bit manipulation, zero HTTP awareness.
- **`src/server/`** — a thin actix-web REST wrapper around `core`.
- **`src/main.rs`** — parses `Cli`, builds `FurDB::new(&furdb_config)` (creates the workdir if missing), and hands it to `Server::start()`. The single `FurDB` is shared with handlers as `web::Data<FurDB>`.

Modules follow the `foo.rs` + `foo/` sibling-directory convention (no `mod.rs`).

### The `models` / `operations` split

Types live in `models/`; behaviour lives in `operations/` as **one `impl` block per file**. E.g. `Table` is declared in `src/core/models/table.rs`, but `Table::query`, `Table::insert_entries`, and `Table::generate_sortfile` each live in their own file under `src/core/operations/table/`.

To add a core operation: create `src/core/operations/<entity>/<op>.rs` with an `impl <Type> { pub fn <op> ... }`, then declare the module in `src/core/operations/<entity>.rs`. The parent `src/core/operations.rs` only declares `database`, `furdb`, and `table` — nothing is re-exported, since the methods attach to the types themselves.

The entity hierarchy is `FurDB` → `Database` → `Table`, and each level's operations directory mirrors it: `operations/furdb/` creates/gets/deletes databases, `operations/database/` creates/gets/deletes tables, `operations/table/` handles entries.

### On-disk layout

```
<workdir>/
  <database_id>/
    database_config.json          # serialized DatabaseInfo
    tables/
      <table_id>/
        table_config.json         # serialized TableInfo (incl. column sizes)
        data                      # bit-packed entries, no header
        sortfile                  # per-column sorted index
```

**Always** derive paths through the helpers in `src/core/utils.rs` (`get_table_data_path`, `get_sortfile_path`, …) rather than joining strings. Database/table IDs are validated by `utils::is_id_valid` (alphanumeric plus `-` and `_`); each operation calls it independently, and it is the only guard against path traversal — note `char::is_alphanumeric` is Unicode-aware, so non-ASCII letters pass.

### Bit-packed storage

Each `Column` declares a bit `size` (`u128` values). A row is the concatenation of its columns' bits, MSB-first (`BitVec<u8, Msb0>`). **The total row size must be a multiple of 8** — `create_table` rejects anything else with `TableCreationError::ColumnsUnfit`, because the whole engine computes `entry_size = sum(column sizes) / 8` bytes and addresses entries as `index * entry_size` byte offsets in `data`. Values that don't fit their column are rejected as `EntryInsertionError::ColumnOverflow`. There is no header and no per-entry delimiter; entry count is inferred from file length.

### Sortfile and querying

`sortfile` is a secondary index: for each column, the entry indices sorted by that column's value, each index packed into `identifier_size = (1 + (n-1)/256) * 8` bits. `Table::query(column_index, value)` binary-searches the lower and upper bound within that column's slice of the sortfile and then materializes the matching entries.

Consequences worth knowing before changing write paths:

- `generate_sortfile()` rebuilds the **entire** index from scratch and is called after every insert and delete.
- `delete_entries(indices)` reads all surviving entries into memory, truncates `data`, and re-inserts them — deletes are O(n) rewrites and **renumber every subsequent index**.

### Error handling

`src/core/error.rs` defines a `thiserror` enum per operation kind (`DatabaseReadError`, `TableCreationError`, `EntryInsertionError`, …). `src/server/error.rs` maps each variant into an `ErrorResponse` (`NotFound` / `BadRequest` / `Conflict` / `InternalServerError`) via `From` impls, which is what lets handlers just use `?` on core calls.

**Adding a variant to any core error enum requires updating the matching `From` impl in `src/server/error.rs`,** or the build breaks. Internal details are deliberately swallowed: every `OtherError(_)` maps to a bare `InternalServerError`.

### Response envelope

Every response — success or error — is wrapped by `ApiResponseSerializable` in `src/server/models/response/api_response.rs`:

```json
{ "result": "success", "statusCode": 200, "status": "OK", "response": {} }
```

HTTP status is derived from the enum variant, not chosen in the handler: `SuccessResponse` implements `Responder` and `ErrorResponse` implements `ResponseError`, both delegating to `generate_success` / `generate_error`. So a new endpoint means adding a `SuccessResponse` variant **and** its status-code arm in `api_response.rs`.

Both response enums are `#[serde(untagged)]`, so the variant name never appears in JSON. All models use `#[serde(rename_all = "camelCase")]`; request bodies are typed structs in `src/server/models/params/`.

### Handler conventions

Handlers live one-per-file in `src/server/operations/<group>/`, use actix route macros (`#[get("/{database_id}/{table_id}/data")]`), and must be registered in `Server::start()` in `src/server/furdb_server.rs` — registration is manual and easy to forget.

Routes are `/{database_id}`, `/{database_id}/{table_id}`, and `/{database_id}/{table_id}/data`. Note that the entry-data `GET` and `DELETE` endpoints **take a JSON body** (`GetEntriesParams` / `DeleteEntriesParams`), which many HTTP clients won't send by default — use `curl -X GET -d '...' -H 'Content-Type: application/json'` when testing. See README.md for full request/response examples.

## Platform constraint

`src/core/operations/table/get_entries.rs` uses `std::os::unix::fs::FileExt::read_exact_at`, so the crate builds on Unix only. Preserve or explicitly replace that if portability matters.
