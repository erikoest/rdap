# Object Methods Refactor — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor free functions into object methods on `Formatter`, `Client`, and response types.

**Architecture:** Three objects: `Formatter { nc }` owns all ANSI output helpers; `Client` owns the HTTP client, server URL, and request config plus carries a `Formatter`; response types gain a `print(&self, fmt: &Formatter)` method so lookup/search methods reduce to fetch + print. `impl Client` blocks in `lookup.rs`/`search.rs` extend `Client` without moving code to `client.rs`.

**Tech Stack:** Rust 2021, clap 4, reqwest 0.12, tokio, serde/serde_json, anyhow, url 2.

## Global Constraints

- `cargo build` must succeed with zero warnings after every task.
- No behaviour changes — same CLI flags, same output format, same error messages.
- No new dependencies.
- All field access across modules uses `pub(crate)` visibility, not `pub`.

---

### Task 1: `Formatter` struct and response `print` methods

**Files:**
- Modify: `src/format.rs`

**Interfaces:**
- Produces:
  - `pub struct Formatter { nc: bool }` with `pub fn new(nc: bool) -> Self`
  - Methods: `pub fn heading(&self, s: &str)`, `pub fn row(&self, key: &str, value: &str)`, `pub fn print_events(&self, events: &[Event])`, `pub fn print_entities(&self, entities: &[Entity])`, `pub fn print_next_cursor(&self, links: &[Link])`, `pub fn print_paging_metadata(&self, pm: &PagingMetadata, links: Option<&[Link]>)`, `pub fn print_notices(&self, notices: &[Notice])`
  - Private helpers: `fn dim(&self) -> &'static str`, `fn label(&self, s: &str) -> String`, `fn print_notice(&self, notice: &Notice)`
  - Free function kept: `pub fn extract_cursor(href: &str) -> Option<String>` (no state needed)
  - `impl DomainResponse { pub fn print(&self, fmt: &Formatter) }`
  - `impl HostResponse { pub fn print(&self, fmt: &Formatter) }`
  - `impl EntityResponse { pub fn print(&self, fmt: &Formatter) }`
  - `impl HelpResponse { pub fn print(&self, fmt: &Formatter) }`
  - `impl DomainSearchResponse { pub fn print(&self, fmt: &Formatter) }`
  - `impl EntitySearchResponse { pub fn print(&self, fmt: &Formatter) }`
  - `impl HostSearchResponse { pub fn print(&self, fmt: &Formatter) }`
  - Old free functions (`heading`, `row`, `dim`, `label`, `print_events`, `print_entities`, `print_next_cursor`, `print_paging_metadata`, `print_notices`) **kept as-is** so `lookup.rs`/`search.rs` still compile — they are removed in Task 3.

- [ ] **Step 1: Replace `src/format.rs` with new content**

```rust
use crate::types::{
    DomainResponse, DomainSearchResponse, Entity, EntityResponse, EntitySearchResponse,
    Event, HelpResponse, HostResponse, HostSearchResponse, Link, Notice, PagingMetadata,
    vcard_field,
};

pub const RULE: &str = "──────────────────────────────────────────────────";

// ── Formatter ────────────────────────────────────────────────────────────────

pub struct Formatter {
    nc: bool,
}

impl Formatter {
    pub fn new(nc: bool) -> Self {
        Self { nc }
    }

    pub(crate) fn dim(&self) -> &'static str {
        if self.nc { "\x1b[3m" } else { "\x1b[2m" }
    }

    fn label(&self, s: &str) -> String {
        let esc = if self.nc { "\x1b[1m" } else { "\x1b[1;36m" };
        format!("{}{:<18}\x1b[0m", esc, s)
    }

    pub fn heading(&self, s: &str) {
        let esc = if self.nc { "\x1b[1;4m" } else { "\x1b[1;33m" };
        println!("\n{}{}\x1b[0m", esc, s);
        println!("{}", RULE);
    }

    pub fn row(&self, key: &str, value: &str) {
        println!("  {}  {}", self.label(key), value);
    }

    pub fn print_events(&self, events: &[Event]) {
        for e in events {
            let date = e.date.split('T').next().unwrap_or(&e.date);
            self.row(&e.action, date);
        }
    }

    pub fn print_entities(&self, entities: &[Entity]) {
        for e in entities {
            let roles = e.roles.as_deref().unwrap_or_default().join(", ");
            let handle = e.handle.as_deref().unwrap_or("—");
            let name = vcard_field(&e.vcard_array, "fn").unwrap_or_default();
            let email = vcard_field(&e.vcard_array, "email").unwrap_or_default();
            let mut parts = vec![handle.to_string()];
            if !name.is_empty() { parts.push(name); }
            if !email.is_empty() { parts.push(format!("<{}>", email)); }
            self.row(&roles, &parts.join("  "));
        }
    }

    fn print_notice(&self, notice: &Notice) {
        if let Some(title) = &notice.title {
            println!("\n  {}{}\x1b[0m", self.dim(), title);
        }
        if let Some(lines) = &notice.description {
            for line in lines {
                println!("  {}{}\x1b[0m", self.dim(), line);
            }
        }
    }

    pub fn print_notices(&self, notices: &[Notice]) {
        for notice in notices {
            self.print_notice(notice);
        }
    }

    pub fn print_next_cursor(&self, links: &[Link]) {
        if let Some(next) = links.iter().find(|l| l.rel.as_deref() == Some("next")) {
            if let Some(href) = &next.href {
                if let Some(cursor) = extract_cursor(href) {
                    println!("\n  {}Next page:  --cursor {}\x1b[0m", self.dim(), cursor);
                } else {
                    println!("\n  {}Next:  {}\x1b[0m", self.dim(), href);
                }
            }
        }
    }

    pub fn print_paging_metadata(&self, pm: &PagingMetadata, links: Option<&[Link]>) {
        if let Some(count) = pm.total_count {
            self.row("Total", &count.to_string());
        }
        match (pm.page_number, pm.page_size) {
            (Some(n), Some(s)) => self.row("Page", &format!("{} (size {})", n, s)),
            (Some(n), None)    => self.row("Page", &n.to_string()),
            (None, Some(s))    => self.row("Page size", &s.to_string()),
            (None, None)       => {}
        }
        if let Some(links) = links {
            self.print_next_cursor(links);
        }
    }
}

// ── cursor helper (no state needed) ──────────────────────────────────────────

pub fn extract_cursor(href: &str) -> Option<String> {
    url::Url::parse(href).ok()?
        .query_pairs()
        .find(|(k, _)| k == "cursor")
        .map(|(_, v)| v.into_owned())
}

// ── response print methods ────────────────────────────────────────────────────

impl DomainResponse {
    pub fn print(&self, fmt: &Formatter) {
        if let Some(h) = &self.handle { fmt.row("Handle", h); }
        if let Some(n) = self.unicode_name.as_ref().or(self.ldh_name.as_ref()) {
            fmt.row("Name", n);
        }
        if let Some(statuses) = &self.status {
            fmt.row("Status", &statuses.join(", "));
        }
        if let Some(ns) = &self.nameservers {
            let names: Vec<&str> = ns.iter().filter_map(|n| n.ldh_name.as_deref()).collect();
            fmt.row("Nameservers", &names.join(", "));
        }
        if let Some(events) = &self.events {
            println!();
            fmt.print_events(events);
        }
        if let Some(entities) = &self.entities {
            println!();
            fmt.row("Contacts", "");
            fmt.print_entities(entities);
        }
    }
}

impl HostResponse {
    pub fn print(&self, fmt: &Formatter) {
        if let Some(h) = &self.handle { fmt.row("Handle", h); }
        if let Some(n) = &self.name { fmt.row("Name", n); }
        if let (Some(start), Some(end)) = (&self.start_address, &self.end_address) {
            fmt.row("Range", &format!("{} – {}", start, end));
        }
        if let Some(t) = &self.ip_type { fmt.row("Type", t); }
        if let Some(c) = &self.country { fmt.row("Country", c); }
        if let Some(statuses) = &self.status {
            fmt.row("Status", &statuses.join(", "));
        }
        if let Some(events) = &self.events {
            println!();
            fmt.print_events(events);
        }
        if let Some(entities) = &self.entities {
            println!();
            fmt.row("Contacts", "");
            fmt.print_entities(entities);
        }
    }
}

impl EntityResponse {
    pub fn print(&self, fmt: &Formatter) {
        if let Some(h) = &self.handle { fmt.row("Handle", h); }
        if let Some(c) = &self.class { fmt.row("Class", c); }
        if let Some(roles) = &self.roles { fmt.row("Roles", &roles.join(", ")); }
        if let Some(name) = vcard_field(&self.vcard_array, "fn") { fmt.row("Name", &name); }
        if let Some(org) = vcard_field(&self.vcard_array, "org") { fmt.row("Organisation", &org); }
        if let Some(email) = vcard_field(&self.vcard_array, "email") { fmt.row("Email", &email); }
        if let Some(tel) = vcard_field(&self.vcard_array, "tel") { fmt.row("Phone", &tel); }
        if let Some(statuses) = &self.status {
            fmt.row("Status", &statuses.join(", "));
        }
        if let Some(events) = &self.events {
            println!();
            fmt.print_events(events);
        }
        if let Some(entities) = &self.entities {
            println!();
            fmt.row("Contacts", "");
            fmt.print_entities(entities);
        }
    }
}

impl HelpResponse {
    pub fn print(&self, fmt: &Formatter) {
        for notice in self.notices.iter().flatten().chain(self.remarks.iter().flatten()) {
            fmt.print_notice(notice);
        }
    }
}

impl DomainSearchResponse {
    pub fn print(&self, fmt: &Formatter) {
        let results = self.results.as_deref().unwrap_or_default();
        if results.is_empty() {
            println!("  No results.");
        } else {
            for d in results {
                let n = d.unicode_name.as_ref().or(d.ldh_name.as_ref())
                    .map(|s| s.as_str()).unwrap_or("—");
                let status = d.status.as_deref().unwrap_or_default().join(", ");
                println!("  {:<42}  {}", n.to_uppercase(), status);
            }
        }
        if let Some(pm) = &self.paging_metadata {
            let next_links = pm.links.as_deref().filter(|l| !l.is_empty())
                .or(self.links.as_deref());
            fmt.print_paging_metadata(pm, next_links);
        } else if let Some(links) = self.links.as_deref() {
            fmt.print_next_cursor(links);
        }
        if let Some(notices) = self.notices.as_deref() {
            fmt.print_notices(notices);
        }
    }
}

impl EntitySearchResponse {
    pub fn print(&self, fmt: &Formatter) {
        let results = self.results.as_deref().unwrap_or_default();
        if results.is_empty() {
            println!("  No results.");
        } else {
            for e in results {
                let h = e.handle.as_deref().unwrap_or("—");
                let roles = e.roles.as_deref().unwrap_or_default().join(", ");
                let name = vcard_field(&e.vcard_array, "fn").unwrap_or_default();
                println!("  {:<24}  {:<20}  {}", h, roles, name);
            }
        }
        if let Some(pm) = &self.paging_metadata {
            let next_links = pm.links.as_deref().filter(|l| !l.is_empty())
                .or(self.links.as_deref());
            fmt.print_paging_metadata(pm, next_links);
        } else if let Some(links) = self.links.as_deref() {
            fmt.print_next_cursor(links);
        }
        if let Some(notices) = self.notices.as_deref() {
            fmt.print_notices(notices);
        }
    }
}

impl HostSearchResponse {
    pub fn print(&self, fmt: &Formatter) {
        let results = self.results.as_deref().unwrap_or_default();
        if results.is_empty() {
            println!("  No results.");
        } else {
            for ns in results {
                println!("  {}", ns.ldh_name.as_deref().unwrap_or("—"));
            }
        }
        if let Some(pm) = &self.paging_metadata {
            let next_links = pm.links.as_deref().filter(|l| !l.is_empty())
                .or(self.links.as_deref());
            fmt.print_paging_metadata(pm, next_links);
        } else if let Some(links) = self.links.as_deref() {
            fmt.print_next_cursor(links);
        }
        if let Some(notices) = self.notices.as_deref() {
            fmt.print_notices(notices);
        }
    }
}

// ── legacy free functions (kept for compilation; removed in Task 3) ───────────

pub fn dim(nc: bool) -> &'static str { if nc { "\x1b[3m" } else { "\x1b[2m" } }
pub fn label(s: &str, nc: bool) -> String {
    let esc = if nc { "\x1b[1m" } else { "\x1b[1;36m" };
    format!("{}{:<18}\x1b[0m", esc, s)
}
pub fn heading(s: &str, nc: bool) {
    let esc = if nc { "\x1b[1;4m" } else { "\x1b[1;33m" };
    println!("\n{}{}\x1b[0m", esc, s);
    println!("{}", RULE);
}
pub fn row(key: &str, value: &str, nc: bool) { println!("  {}  {}", label(key, nc), value); }
pub fn print_events(events: &[Event], nc: bool) {
    for e in events { let date = e.date.split('T').next().unwrap_or(&e.date); row(&e.action, date, nc); }
}
pub fn print_entities(entities: &[Entity], nc: bool) {
    for e in entities {
        let roles = e.roles.as_deref().unwrap_or_default().join(", ");
        let handle = e.handle.as_deref().unwrap_or("—");
        let name = vcard_field(&e.vcard_array, "fn").unwrap_or_default();
        let email = vcard_field(&e.vcard_array, "email").unwrap_or_default();
        let mut parts = vec![handle.to_string()];
        if !name.is_empty() { parts.push(name); }
        if !email.is_empty() { parts.push(format!("<{}>", email)); }
        row(&roles, &parts.join("  "), nc);
    }
}
pub fn print_next_cursor(links: &[Link], nc: bool) {
    if let Some(next) = links.iter().find(|l| l.rel.as_deref() == Some("next")) {
        if let Some(href) = &next.href {
            if let Some(cursor) = extract_cursor(href) {
                println!("\n  {}Next page:  --cursor {}\x1b[0m", dim(nc), cursor);
            } else {
                println!("\n  {}Next:  {}\x1b[0m", dim(nc), href);
            }
        }
    }
}
pub fn print_paging_metadata(pm: &PagingMetadata, links: Option<&[Link]>, nc: bool) {
    if let Some(count) = pm.total_count { row("Total", &count.to_string(), nc); }
    match (pm.page_number, pm.page_size) {
        (Some(n), Some(s)) => row("Page", &format!("{} (size {})", n, s), nc),
        (Some(n), None)    => row("Page", &n.to_string(), nc),
        (None, Some(s))    => row("Page size", &s.to_string(), nc),
        (None, None)       => {}
    }
    if let Some(links) = links { print_next_cursor(links, nc); }
}
pub fn print_notices(notices: &[Notice], nc: bool) {
    for notice in notices {
        if let Some(title) = &notice.title { println!("\n  {}{}\x1b[0m", dim(nc), title); }
        if let Some(lines) = &notice.description {
            for line in lines { println!("  {}{}\x1b[0m", dim(nc), line); }
        }
    }
}
```

- [ ] **Step 2: Verify**

```bash
cargo build
```

Expected: success, zero warnings.

- [ ] **Step 3: Commit**

```bash
git add src/format.rs
git commit -m "refactor: add Formatter struct and response print methods"
```

---

### Task 2: `Client` struct

**Files:**
- Modify: `src/client.rs`

**Interfaces:**
- Consumes: `Formatter::new` from Task 1
- Produces:
  - `pub struct Client` with fields `http`, `server`, `auth`, `cursor`, `count`, `sort`, `fields`, `debug` (all private), `pub(crate) fmt: Formatter`, `pub(crate) server: String`
  - `pub fn Client::new(http, server, auth, cursor, count, sort, fields, debug, no_color) -> Self`
  - `pub(crate) async fn Client::fetch<T: DeserializeOwned>(&self, url: &str, extra: &[(&str,&str)]) -> anyhow::Result<T>`
  - Old `Opts<'a>` struct and `Auth<'a>` type alias and free `fetch` function **kept** so `lookup.rs`/`search.rs` still compile — removed in Task 3.

- [ ] **Step 1: Replace `src/client.rs` with new content**

```rust
use crate::format::Formatter;

// ── Client ───────────────────────────────────────────────────────────────────

pub struct Client {
    http: reqwest::Client,
    pub(crate) server: String,
    auth: Option<(String, String)>,
    cursor: Option<String>,
    count: bool,
    sort: Option<String>,
    fields: Option<String>,
    debug: bool,
    pub(crate) fmt: Formatter,
}

impl Client {
    pub fn new(
        http: reqwest::Client,
        server: String,
        auth: Option<(String, String)>,
        cursor: Option<String>,
        count: bool,
        sort: Option<String>,
        fields: Option<String>,
        debug: bool,
        no_color: bool,
    ) -> Self {
        Self { http, server, auth, cursor, count, sort, fields, debug, fmt: Formatter::new(no_color) }
    }

    pub(crate) async fn fetch<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        extra: &[(&str, &str)],
    ) -> anyhow::Result<T> {
        let mut req = self.http.get(url);
        if let Some((user, pass)) = &self.auth {
            req = req.basic_auth(user, Some(pass.as_str()));
        }
        let mut params: Vec<(&str, &str)> = extra.to_vec();
        if let Some(c) = &self.cursor { params.push(("cursor", c)); }
        if self.count { params.push(("count", "true")); }
        if let Some(s) = &self.sort { params.push(("sort", s)); }
        if let Some(f) = &self.fields { params.push(("fields", f)); }
        if !params.is_empty() { req = req.query(&params); }
        let req = req.build()?;
        if self.debug {
            eprintln!("{}> {}\x1b[0m", self.fmt.dim(), req.url());
        }
        Ok(self.http.execute(req).await?.error_for_status()?.json().await?)
    }
}

// ── legacy Opts / fetch (kept for compilation; removed in Task 3) ─────────────

pub type Auth<'a> = Option<(&'a str, &'a str)>;

pub struct Opts<'a> {
    pub auth: Auth<'a>,
    pub cursor: Option<&'a str>,
    pub count: bool,
    pub sort: Option<&'a str>,
    pub fields: Option<&'a str>,
    pub debug: bool,
    pub no_color: bool,
}

pub async fn fetch<T: serde::de::DeserializeOwned>(
    client: &reqwest::Client,
    url: &str,
    opts: &Opts<'_>,
    extra: &[(&str, &str)],
) -> anyhow::Result<T> {
    let mut req = client.get(url);
    if let Some((user, pass)) = opts.auth { req = req.basic_auth(user, Some(pass)); }
    let mut params: Vec<(&str, &str)> = extra.to_vec();
    if let Some(c) = opts.cursor { params.push(("cursor", c)); }
    if opts.count { params.push(("count", "true")); }
    if let Some(s) = opts.sort { params.push(("sort", s)); }
    if let Some(f) = opts.fields { params.push(("fields", f)); }
    if !params.is_empty() { req = req.query(&params); }
    let req = req.build()?;
    if opts.debug {
        eprintln!("{}> {}\x1b[0m", dim(opts.no_color), req.url());
    }
    Ok(client.execute(req).await?.error_for_status()?.json().await?)
}
```

- [ ] **Step 2: Verify**

```bash
cargo build
```

Expected: success, zero warnings.

- [ ] **Step 3: Commit**

```bash
git add src/client.rs src/format.rs
git commit -m "refactor: add Client struct with owned config and fetch method"
```

---

### Task 3: `impl Client` in lookup/search, update bin, remove legacy code

**Files:**
- Modify: `src/lookup.rs`
- Modify: `src/search.rs`
- Modify: `src/bin/rdap.rs`
- Modify: `src/client.rs` (remove legacy `Opts`, `Auth`, free `fetch`)
- Modify: `src/format.rs` (remove legacy free functions)
- Modify: `src/lib.rs` (make `lookup`/`search` private mods)

**Interfaces:**
- Consumes: `Client::new`, `Client::fetch`, `Client::server`, `Client::fmt`, all response `print` methods from Task 1
- Produces: `client.lookup_domain(name)`, `client.lookup_host(address)`, `client.lookup_entity(handle)`, `client.lookup_help()`, `client.search_domains(name)`, `client.search_entities(handle, fn_name)`, `client.search_hosts(name, ip)` — all `pub async fn` on `Client`

- [ ] **Step 1: Replace `src/lookup.rs`**

```rust
use crate::client::Client;
use crate::types::{DomainResponse, EntityResponse, HelpResponse, HostResponse};

impl Client {
    pub async fn lookup_domain(&self, name: &str) -> anyhow::Result<()> {
        let url = format!("{}/domain/{}", self.server, name);
        let resp: DomainResponse = self.fetch(&url, &[]).await?;
        self.fmt.heading(&format!("Domain: {}", name.to_uppercase()));
        resp.print(&self.fmt);
        println!();
        Ok(())
    }

    pub async fn lookup_host(&self, address: &str) -> anyhow::Result<()> {
        let url = format!("{}/ip/{}", self.server, address);
        let resp: HostResponse = self.fetch(&url, &[]).await?;
        self.fmt.heading(&format!("Host / Network: {}", address));
        resp.print(&self.fmt);
        println!();
        Ok(())
    }

    pub async fn lookup_entity(&self, handle: &str) -> anyhow::Result<()> {
        let url = format!("{}/entity/{}", self.server, handle);
        let resp: EntityResponse = self.fetch(&url, &[]).await?;
        self.fmt.heading(&format!("Entity: {}", handle));
        resp.print(&self.fmt);
        println!();
        Ok(())
    }

    pub async fn lookup_help(&self) -> anyhow::Result<()> {
        let url = format!("{}/help", self.server);
        let resp: HelpResponse = self.fetch(&url, &[]).await?;
        self.fmt.heading(&format!("Help: {}", self.server));
        resp.print(&self.fmt);
        println!();
        Ok(())
    }
}
```

- [ ] **Step 2: Replace `src/search.rs`**

```rust
use crate::client::Client;
use crate::types::{DomainSearchResponse, EntitySearchResponse, HostSearchResponse};

impl Client {
    pub async fn search_domains(&self, name: &str) -> anyhow::Result<()> {
        let url = format!("{}/domains", self.server);
        let resp: DomainSearchResponse = self.fetch(&url, &[("name", name)]).await?;
        self.fmt.heading(&format!("Domains matching \"{}\"", name));
        resp.print(&self.fmt);
        println!();
        Ok(())
    }

    pub async fn search_entities(
        &self,
        handle: Option<&str>,
        fn_name: Option<&str>,
    ) -> anyhow::Result<()> {
        let url = format!("{}/entities", self.server);
        let extra: Vec<(&str, &str)> = match (handle, fn_name) {
            (Some(h), _) => vec![("handle", h)],
            (_, Some(f)) => vec![("fn", f)],
            _ => anyhow::bail!("entities search requires --handle or --fn"),
        };
        let resp: EntitySearchResponse = self.fetch(&url, &extra).await?;
        let pattern = handle.or(fn_name).unwrap_or("*");
        self.fmt.heading(&format!("Entities matching \"{}\"", pattern));
        resp.print(&self.fmt);
        println!();
        Ok(())
    }

    pub async fn search_hosts(
        &self,
        name: Option<&str>,
        ip: Option<&str>,
    ) -> anyhow::Result<()> {
        let url = format!("{}/hosts", self.server);
        let extra: Vec<(&str, &str)> = match (name, ip) {
            (Some(n), _) => vec![("name", n)],
            (_, Some(i)) => vec![("ip", i)],
            _ => anyhow::bail!("hosts search requires --name or --ip"),
        };
        let resp: HostSearchResponse = self.fetch(&url, &extra).await?;
        let pattern = name.or(ip).unwrap_or("*");
        self.fmt.heading(&format!("Nameservers matching \"{}\"", pattern));
        resp.print(&self.fmt);
        println!();
        Ok(())
    }
}
```

- [ ] **Step 3: Replace `src/bin/rdap.rs`**

```rust
use clap::{Parser, Subcommand};
use rdap::client::Client;

#[derive(Parser)]
#[command(name = "rdap", about = "RDAP lookup client", disable_help_subcommand = true)]
struct Cli {
    /// RDAP server base URL (default: https://rdap.org)
    #[arg(short, long, value_name = "URL")]
    server: Option<String>,
    /// Force connections over IPv4
    #[arg(long, conflicts_with = "ipv6")]
    ipv4: bool,
    /// Force connections over IPv6
    #[arg(long, conflicts_with = "ipv4")]
    ipv6: bool,
    /// Username for HTTP Basic authentication
    #[arg(short, long, requires = "password")]
    user: Option<String>,
    /// Password for HTTP Basic authentication
    #[arg(short, long, requires = "user")]
    password: Option<String>,
    /// Continue paging from this cursor (RFC-8977)
    #[arg(long, value_name = "CURSOR")]
    cursor: Option<String>,
    /// Request a count of matching objects in the response (RFC-8977)
    #[arg(long)]
    count: bool,
    /// Sort field; prefix with - for descending order (RFC-8977)
    #[arg(long, value_name = "FIELD")]
    sort: Option<String>,
    /// Comma-separated top-level fields to return (RFC-8982)
    #[arg(long, value_name = "FIELDS")]
    fields: Option<String>,
    /// Print the request URL before sending
    #[arg(long)]
    debug: bool,
    /// Use bold/underline/italic instead of colors
    #[arg(long)]
    no_color: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Look up a domain name
    Domain { name: String },
    /// Look up an IP address or CIDR block
    Host { address: String },
    /// Look up an entity by handle
    Entity { handle: String },
    /// Search for domain names
    Domains { name: String },
    /// Search for entities
    Entities {
        #[arg(long, value_name = "PATTERN")]
        handle: Option<String>,
        #[arg(long = "fn", value_name = "PATTERN")]
        fn_name: Option<String>,
    },
    /// Search for hosts/nameservers
    Hosts {
        #[arg(long, value_name = "PATTERN")]
        name: Option<String>,
        #[arg(long, value_name = "PATTERN")]
        ip: Option<String>,
    },
    /// Fetch server help and usage notices
    Help,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let local_addr: Option<std::net::IpAddr> = if cli.ipv4 {
        Some(std::net::Ipv4Addr::UNSPECIFIED.into())
    } else if cli.ipv6 {
        Some(std::net::Ipv6Addr::UNSPECIFIED.into())
    } else {
        None
    };

    let mut builder = reqwest::Client::builder().user_agent("rdap-cli/0.1");
    if let Some(addr) = local_addr {
        builder = builder.local_address(addr);
    }
    let http = builder.build().expect("failed to build HTTP client");

    let server = cli.server.as_deref().unwrap_or("https://rdap.org")
        .trim_end_matches('/').to_string();
    let auth = cli.user.zip(cli.password);

    let client = Client::new(
        http, server, auth, cli.cursor, cli.count,
        cli.sort, cli.fields, cli.debug, cli.no_color,
    );

    let result = match &cli.command {
        Command::Domain { name }    => client.lookup_domain(name).await,
        Command::Host { address }   => client.lookup_host(address).await,
        Command::Entity { handle }  => client.lookup_entity(handle).await,
        Command::Domains { name }   => client.search_domains(name).await,
        Command::Entities { handle, fn_name } =>
            client.search_entities(handle.as_deref(), fn_name.as_deref()).await,
        Command::Hosts { name, ip } =>
            client.search_hosts(name.as_deref(), ip.as_deref()).await,
        Command::Help               => client.lookup_help().await,
    };

    if let Err(e) = result {
        let esc = if cli.no_color { "\x1b[1m" } else { "\x1b[1;31m" };
        eprintln!("{}error:\x1b[0m {}", esc, e);
        std::process::exit(1);
    }
}
```

- [ ] **Step 4: Remove legacy code from `src/client.rs`**

Replace the entire file with the clean version (no `Opts`, `Auth`, or free `fetch`):

```rust
use crate::format::Formatter;

pub struct Client {
    http: reqwest::Client,
    pub(crate) server: String,
    auth: Option<(String, String)>,
    cursor: Option<String>,
    count: bool,
    sort: Option<String>,
    fields: Option<String>,
    debug: bool,
    pub(crate) fmt: Formatter,
}

impl Client {
    pub fn new(
        http: reqwest::Client,
        server: String,
        auth: Option<(String, String)>,
        cursor: Option<String>,
        count: bool,
        sort: Option<String>,
        fields: Option<String>,
        debug: bool,
        no_color: bool,
    ) -> Self {
        Self { http, server, auth, cursor, count, sort, fields, debug, fmt: Formatter::new(no_color) }
    }

    pub(crate) async fn fetch<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        extra: &[(&str, &str)],
    ) -> anyhow::Result<T> {
        let mut req = self.http.get(url);
        if let Some((user, pass)) = &self.auth {
            req = req.basic_auth(user, Some(pass.as_str()));
        }
        let mut params: Vec<(&str, &str)> = extra.to_vec();
        if let Some(c) = &self.cursor { params.push(("cursor", c)); }
        if self.count { params.push(("count", "true")); }
        if let Some(s) = &self.sort { params.push(("sort", s)); }
        if let Some(f) = &self.fields { params.push(("fields", f)); }
        if !params.is_empty() { req = req.query(&params); }
        let req = req.build()?;
        if self.debug {
            eprintln!("{}> {}\x1b[0m", self.fmt.dim(), req.url());
        }
        Ok(self.http.execute(req).await?.error_for_status()?.json().await?)
    }
}
```

- [ ] **Step 5: Remove legacy free functions from `src/format.rs`**

Delete the entire `// ── legacy free functions` section at the bottom of `src/format.rs` (the `dim`, `label`, `heading`, `row`, `print_events`, `print_entities`, `print_next_cursor`, `print_paging_metadata`, `print_notices` free functions added in Task 1 as temporary shims).

- [ ] **Step 6: Update `src/lib.rs`**

`lookup` and `search` no longer export public items — they only add `impl Client` blocks. Make them private:

```rust
pub mod client;
pub mod format;
mod lookup;
mod search;
pub mod types;
```

- [ ] **Step 7: Verify**

```bash
cargo build
```

Expected: success, zero warnings.

- [ ] **Step 8: Commit**

```bash
git add src/lookup.rs src/search.rs src/bin/rdap.rs src/client.rs src/format.rs src/lib.rs
git commit -m "refactor: convert lookup/search to impl Client methods, remove Opts"
```
