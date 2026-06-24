# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
cargo build          # debug build
cargo build --release
cargo run -- domain example.com
cargo run -- host 8.8.8.8
cargo run -- entity GOGL
cargo clippy
cargo test
```

## Interface

Support the following RDAP API calls:
- Domain lookup: /domain
- Entity lookup: /entity
- Host lookup: /host
- Domains search: /domains
- Entities search: /entities
- Hosts search: /hosts
- Help page: /help

Support the following RDAP extensions:
- Basic authentication: RFC-2617
- Paging and sorting: RFC-8977
- Partial response: RFC-8982

## Architecture

Rust library + binary (`src/lib.rs` + `src/bin/rdap.rs`) built with:
- **clap** (derive) — subcommands `domain`, `host`, `entity`, `domains`, `entities`, `hosts`, `help` plus global flags `--server`, `--ipv4`/`--ipv6`, `--user`/`--password`, `--cursor`, `--count`, `--sort`, `--fields`, `--debug`, `--no-color`
- **reqwest** + **tokio** — async HTTP with optional `local_address` binding for IP version forcing and per-request `.basic_auth()`
- **serde/serde_json** — typed response structs (`DomainResponse`, `HostResponse`, `EntityResponse`) with `Option` fields throughout since RDAP fields are all optional per spec

### Module layout

```
src/
  lib.rs       — pub mod declarations
  types.rs     — pure data structs (serde only, no display logic)
  format.rs    — Formatter struct + impl display blocks on response types
  client.rs    — Client struct + private fetch method
  lookup.rs    — impl Client: lookup_domain/host/entity/help
  search.rs    — impl Client: search_domains/entities/hosts
  bin/rdap.rs  — parse Cli, build Client, dispatch one method call
```

### Key objects

**`Formatter { nc: bool }`** (`src/format.rs`) — wraps the `--no-color` flag. All ANSI output goes through its methods (`heading`, `row`, `print_events`, `print_entities`, `print_notices`, `print_paging_metadata`, `print_next_cursor`). Each response type has a `print(&self, fmt: &Formatter)` method defined here via `impl` blocks.

**`Client`** (`src/client.rs`) — owns the `reqwest::Client`, server URL, auth credentials, and all RFC-8977/8982 request params. `fetch<T>()` is a private async method; lookup and search methods are added via `impl Client` blocks in `lookup.rs` and `search.rs`. Built once in `main` and passed by reference.

### Key patterns

RDAP contacts use jCard (`vcardArray`) encoding — `vcard_field()` walks the nested `["vcard", [[type, params, kind, value], ...]]` structure to extract named fields.

RFC-8977 paging: search responses deserialise a `pagingMetadata` block (`totalCount`, `pageNumber`, `pageSize`, `links`). The next-page cursor is extracted from the `rel=next` link href, percent-decoded via `url::Url::query_pairs()`, and shown as `--cursor <value>`.

Default server is `https://rdap.org` (a bootstrapping proxy that redirects to the authoritative registry). Trailing slashes on `--server` are stripped before URL construction.
