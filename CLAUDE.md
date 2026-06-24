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

Single-file Rust CLI (`src/main.rs`) built with:
- **clap** (derive) — subcommands `domain`, `host`, `entity` plus global flags `--server`, `--ipv4`/`--ipv6`, `--user`/`--password`
- **reqwest** + **tokio** — async HTTP; one shared `reqwest::Client` built in `main` with optional `local_address` binding for IP version forcing and per-request `.basic_auth()`
- **serde/serde_json** — typed response structs (`DomainResponse`, `HostResponse`, `EntityResponse`) with `Option` fields throughout since RDAP fields are all optional per spec

### Key patterns

All HTTP goes through `fetch<T>()` — a generic helper that applies auth and deserialises the response. Lookup functions (`lookup_domain`, `lookup_host`, `lookup_entity`) take `(client, auth, server, target)` and call `fetch`.

RDAP contacts use jCard (`vcardArray`) encoding — `vcard_field()` walks the nested `["vcard", [[type, params, kind, value], ...]]` structure to extract named fields.

Formatting is plain ANSI via `heading()` / `row()` / `label()` helpers — no external terminal crate.

Default server is `https://rdap.org` (a bootstrapping proxy that redirects to the authoritative registry). Trailing slashes on `--server` are stripped before URL construction.
