# Object Methods Refactor — Design

## Goal

Refactor the RDAP CLI from free functions to object methods where natural, reducing parameter threading and clarifying ownership.

## Objects

### `Formatter { nc: bool }` (`src/format.rs`)

Wraps the `no_color` flag. All print/format helpers become methods:

- `heading(&self, s: &str)`
- `row(&self, key: &str, value: &str)`
- `print_events(&self, events: &[Event])`
- `print_entities(&self, entities: &[Entity])`
- `print_notices(&self, notices: &[Notice])`
- `print_paging_metadata(&self, pm: &PagingMetadata, links: Option<&[Link]>)`
- `print_next_cursor(&self, links: &[Link])`

Response display via `impl` blocks on each response type in `format.rs` (avoids circular deps — `format.rs` already imports `types.rs`):

- `DomainResponse::print(&self, fmt: &Formatter)`
- `HostResponse::print(&self, fmt: &Formatter)`
- `EntityResponse::print(&self, fmt: &Formatter)`
- `HelpResponse::print(&self, fmt: &Formatter)`
- `DomainSearchResponse::print(&self, fmt: &Formatter)`
- `EntitySearchResponse::print(&self, fmt: &Formatter)`
- `HostSearchResponse::print(&self, fmt: &Formatter)`

`extract_cursor` stays a free function (no state needed). `dim` and `label` become private helpers or methods.

### `Client` (`src/client.rs`)

Owns everything needed for a request session:

```rust
pub struct Client {
    http: reqwest::Client,
    server: String,
    auth: Option<(String, String)>,
    cursor: Option<String>,
    count: bool,
    sort: Option<String>,
    fields: Option<String>,
    debug: bool,
    pub fmt: Formatter,
}
```

`Opts<'a>` is removed. `fetch<T>` becomes a private method. Constructor: `Client::new(http, server, fmt, ...)` or via `ClientBuilder`.

### `impl Client` in `lookup.rs` and `search.rs`

Each lookup/search function becomes a method. Body: `self.fetch(url, extra).await?` then `resp.print(&self.fmt)`.

## Module layout (unchanged)

```
src/
  lib.rs       — pub mod declarations
  types.rs     — pure data structs (unchanged)
  format.rs    — Formatter + impl display on response types
  client.rs    — Client struct + fetch
  lookup.rs    — impl Client: lookup_*
  search.rs    — impl Client: search_*
  bin/rdap.rs  — parse Cli, build Client, dispatch
```

## Dependency directions (unchanged)

`format` → `types`, `client` → `format`, `lookup`/`search` → `client` + `types`, `bin/rdap` → `client`.

## Non-goals

- No new public API surface beyond the CLI binary
- `types.rs` gets no display imports (stays pure data)
