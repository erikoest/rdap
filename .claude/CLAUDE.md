# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

@RTK.md

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

## Git Guidelines

- NEVER execute git commit, git push, or git merge.
- When you are finished with a task, summarize the changes and ask the user to commit manually.

## Interface

Support the following RDAP API calls:
- Domain lookup: /domain
- Entity lookup: /entity
- Host lookup: /host
- Domains search: /domains
- Entities search: /entities
- Hosts search: /hosts
- Help page: /help

Support the following Norid extensions to API:
- Nameserver handle: /nameserver_handle
- Domain count: /norid_domain_count

Norid extensions (as described here: https://teknisk.norid.no/en/integrere-mot-norid/rdap-tjenesten/#Local-adjustments):
  norid_domain_count subcommand has one argument which is a string with characters [INPR9-9.]
  nameserver_handle subcommand has one argument which is a string with characters [A-Z0-8-]

Support the following RDAP extensions:
- Basic authentication: RFC-2617
- Paging and sorting: RFC-8977
- Partial response: RFC-8982

Support paging for search subcommands:
- If the response contains a paging_metadata containing a 'next' link, this means that only part of the search answer has been returned. In this case the user must have the choice of retrieving the next part, by pressing 'space', or quit the search by pressing 'q'. By pressing space, the search request as shown in the 'next' link must be sent, and the response shown.

Domain lookup must show nameserver handle for each nameserver in output

norid_domain_count subcommand must show domain_count.count and domain_count.parentDomainName in output

Entity lookup must show publicIds whenever this block is present.

Search output must show cursor to next page, page number, total number of hits and page size whenever this is present in the metadata block.

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

**`Formatter { nc: bool }`** (`src/format.rs`) — wraps the `--no-color` flag. All ANSI output goes through its methods (`heading`, `row`, `print_events`, `print
_entities`, `print_notices`, `print_paging_metadata`, `print_next_cursor`). Each response type has a `print(&self, fmt: &Formatter)` method defined here via `impl` blocks.

**`Client`** (`src/client.rs`) — owns the `reqwest::Client`, server URL, auth credentials, and all RFC-8977/8982 request params. `fetch<T>()` is a private async method; lookup and search methods are added via `impl Client` blocks in `lookup.rs` and `search.rs`. Built once in `main` and passed by reference.

### Key patterns

RDAP contacts use jCard (`vcardArray`) encoding — `vcard_field()` walks the nested `["vcard", [[type, params, kind, value], ...]]` structure to extract named fields.

RFC-8977 paging: search responses deserialise a `pagingMetadata` block (`totalCount`, `pageNumber`, `pageSize`, `links`). The next-page cursor is extracted from the `rel=next` link href, percent-decoded via `url::Url::query_pairs()`, and shown as `--cursor <value>`.

Default server is `https://rdap.org` (a bootstrapping proxy that redirects to the authoritative registry). Trailing slashes on `--server` are stripped before URL construction.
