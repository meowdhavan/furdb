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

Tests live in `#[cfg(test)] mod tests` inside `src/server/furdb_service.rs` and drive the gRPC service in-process (`cargo test`, or `cargo test <name>` for one). The crate is binary-only, so a `tests/` directory cannot `use furdb::…` — new tests belong inside `src/`.

`build.rs` compiles `proto/furdb.proto` with `tonic-prost-build`. `protoc` is vendored via `protoc-bin-vendored`, so no system package is needed; the generated module is pulled in by `tonic::include_proto!` in `src/server/proto.rs`.

`WORKDIR` and `PORT` env vars back the `--workdir` / `--port` flags (clap `env` attribute), and `main.rs` loads `.env` via `dotenv`. Verbosity comes from `clap-verbosity-flag` (`-v`, `-vv`, …), defaulting to info.

CI: `.github/workflows/docker-image.yml` builds and pushes the Docker image; `publish-furdb-crate.yml` publishes to crates.io and is `workflow_dispatch`-only.

## Architecture

Single binary crate with a strict two-layer split:

- **`src/core/`** — the storage engine. Pure filesystem + bit manipulation, zero transport awareness.
- **`src/server/`** — a thin tonic gRPC wrapper around `core`.
- **`src/main.rs`** — parses `Cli`, builds `FurDB::new(&furdb_config)` (creates the workdir if missing), and hands it to `Server::start()`. The single `FurDB` is owned by `FurDbService` and shared with handlers by reference.

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

Each `Column` declares a bit `size` (`u128` values). A row is the concatenation of its columns' bits, MSB-first (`BitVec<u8, Msb0>`). **The total row size must be a non-zero multiple of 8** — `create_table` rejects anything else with `TableCreationError::ColumnsUnfit`, because the whole engine computes `entry_size = sum(column sizes) / 8` bytes and addresses entries as `index * entry_size` byte offsets in `data`. Values that don't fit their column are rejected as `EntryInsertionError::ColumnOverflow`. There is no header and no per-entry delimiter; entry count is inferred from file length.

### Sortfile and querying

`sortfile` is a secondary index: for each column, the entry indices sorted by that column's value, each index packed into `identifier_size = (1 + (n-1)/256) * 8` bits. `Table::query(column_index, value)` binary-searches the lower and upper bound within that column's slice of the sortfile and then materializes the matching entries.

Consequences worth knowing before changing write paths:

- `generate_sortfile()` rebuilds the **entire** index from scratch and is called after every insert and delete.
- `delete_entries(indices)` reads all surviving entries into memory, truncates `data`, and re-inserts them — deletes are O(n) rewrites and **renumber every subsequent index**.

### Error handling

`src/core/error.rs` defines a `thiserror` enum per operation kind (`DatabaseReadError`, `TableCreationError`, `EntryInsertionError`, …). `src/server/error.rs` maps each variant into an `ErrorResponse` (`NotFound` / `BadRequest` / `Conflict` / `InternalServerError`) via `From` impls, which is what lets handlers just use `?` on core calls.

**Adding a variant to any core error enum requires updating the matching `From` impl in `src/server/error.rs`,** or the build breaks. Internal details are deliberately swallowed: every `OtherError(_)` maps to a bare `InternalServerError`.

### Response envelope

Success responses keep the envelope the REST API used to return — every `*Response` message in `proto/furdb.proto` declares `result`, `status_code`, `status` and (where there is a payload) `response`:

```json
{ "result": "success", "statusCode": 200, "status": "OK", "response": {} }
```

The status is decided in one place, not in the handler: `src/server/models/response/success_response.rs` invokes the `success_response!` macro once per response message with the `SuccessStatus` it reports. **A new RPC means a new `success_response!` line there**, or the message has no constructor.

Failures do *not* use the envelope — `From<ErrorResponse> for tonic::Status` in `error_response.rs` maps `NotFound`/`BadRequest`/`Conflict`/`InternalServerError` onto `NOT_FOUND`/`INVALID_ARGUMENT`/`ALREADY_EXISTS`/`INTERNAL`, echoing the old HTTP status pair in the `x-furdb-status-code` / `x-furdb-status` trailers.

Core models are converted to their wire form by the `From` impls in `src/server/models/conversions.rs`. **`u128` has no protobuf counterpart**, so entry values and query values cross the wire as decimal strings and are parsed back by `server::utils::parse_u128` (column sizes are bounded by the row size, so they stay `uint64`). The request-side conversions live in `src/server/models/params/`; handlers parse them **before** touching the filesystem so a malformed request is rejected as `INVALID_ARGUMENT` rather than `NOT_FOUND`.

### Handler conventions

Handlers live one-per-file in `src/server/operations/<group>/` as plain functions taking `(&FurDB, proto::<Rpc>Request)` and returning `Result<proto::<Rpc>Response, ErrorResponse>` — they never touch tonic types.

`src/server/furdb_service.rs` holds the single `impl proto::fur_db_server::FurDb for FurDbService` block that every RPC must be listed in — this is the gRPC equivalent of route registration, and it is manual and easy to forget. Its `respond()` helper logs the outcome and converts `ErrorResponse` into `tonic::Status`.

Adding an RPC therefore touches four places: `proto/furdb.proto`, the handler file, `src/server/operations.rs`, and the trait impl in `furdb_service.rs`.

The service is served as h2c (no TLS) on `0.0.0.0:{port}`, and drains in-flight requests on SIGINT/SIGTERM — do not remove that, since a delete killed mid-rewrite leaves `data` and `sortfile` out of step. Note this only covers process termination: nothing locks a table, so concurrent writes to one can still interleave (pre-existing). See README.md for full request/response examples.

## Platform constraint

`src/core/operations/table/get_entries.rs` uses `std::os::unix::fs::FileExt::read_exact_at`, so the crate builds on Unix only. Preserve or explicitly replace that if portability matters.
