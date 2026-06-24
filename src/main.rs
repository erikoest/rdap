use clap::{Parser, Subcommand};
use serde::Deserialize;
use serde_json::Value;

// ── CLI ───────────────────────────────────────────────────────────────────────

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
    /// Max results per page (RFC-8977)
    #[arg(long, value_name = "N")]
    count: Option<u32>,
    /// Sort field; prefix with - for descending order (RFC-8977)
    #[arg(long, value_name = "FIELD")]
    sort: Option<String>,
    /// Comma-separated top-level fields to return (RFC-8982)
    #[arg(long, value_name = "FIELDS")]
    fields: Option<String>,
    /// Print the request URL before sending
    #[arg(long)]
    debug: bool,
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
    Domains {
        /// Name pattern (wildcards: * ?)
        name: String,
    },
    /// Search for entities
    Entities {
        /// Handle pattern
        #[arg(long, value_name = "PATTERN")]
        handle: Option<String>,
        /// Full name (fn) pattern
        #[arg(long = "fn", value_name = "PATTERN")]
        fn_name: Option<String>,
    },
    /// Search for hosts/nameservers
    Hosts {
        /// Name pattern
        #[arg(long, value_name = "PATTERN")]
        name: Option<String>,
        /// IP address pattern
        #[arg(long, value_name = "PATTERN")]
        ip: Option<String>,
    },
    /// Fetch server help and usage notices
    Help,
}

// ── shared types ──────────────────────────────────────────────────────────────

#[derive(Deserialize, Debug)]
struct Notice {
    title: Option<String>,
    description: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
struct Link {
    rel: Option<String>,
    href: Option<String>,
}

#[derive(Deserialize, Debug)]
struct HelpResponse {
    notices: Option<Vec<Notice>>,
    remarks: Option<Vec<Notice>>,
}

#[derive(Deserialize, Debug)]
struct Event {
    #[serde(rename = "eventAction")]
    action: String,
    #[serde(rename = "eventDate")]
    date: String,
}

#[derive(Deserialize, Debug)]
struct Nameserver {
    #[serde(rename = "ldhName")]
    ldh_name: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Entity {
    handle: Option<String>,
    roles: Option<Vec<String>>,
    #[serde(rename = "vcardArray")]
    vcard_array: Option<Value>,
}

// ── lookup response types ─────────────────────────────────────────────────────

#[derive(Deserialize, Debug)]
struct DomainResponse {
    handle: Option<String>,
    #[serde(rename = "ldhName")]
    ldh_name: Option<String>,
    #[serde(rename = "unicodeName")]
    unicode_name: Option<String>,
    status: Option<Vec<String>>,
    events: Option<Vec<Event>>,
    nameservers: Option<Vec<Nameserver>>,
    entities: Option<Vec<Entity>>,
}

#[derive(Deserialize, Debug)]
struct HostResponse {
    handle: Option<String>,
    #[serde(rename = "startAddress")]
    start_address: Option<String>,
    #[serde(rename = "endAddress")]
    end_address: Option<String>,
    name: Option<String>,
    #[serde(rename = "type")]
    ip_type: Option<String>,
    country: Option<String>,
    status: Option<Vec<String>>,
    events: Option<Vec<Event>>,
    entities: Option<Vec<Entity>>,
}

#[derive(Deserialize, Debug)]
struct EntityResponse {
    handle: Option<String>,
    #[serde(rename = "objectClassName")]
    class: Option<String>,
    status: Option<Vec<String>>,
    roles: Option<Vec<String>>,
    #[serde(rename = "vcardArray")]
    vcard_array: Option<Value>,
    events: Option<Vec<Event>>,
    entities: Option<Vec<Entity>>,
}

// ── search response types ─────────────────────────────────────────────────────

#[derive(Deserialize, Debug)]
struct DomainSearchResponse {
    #[serde(rename = "domainSearchResults")]
    results: Option<Vec<DomainResponse>>,
    notices: Option<Vec<Notice>>,
    links: Option<Vec<Link>>,
}

#[derive(Deserialize, Debug)]
struct EntitySearchResponse {
    #[serde(rename = "entitySearchResults")]
    results: Option<Vec<EntityResponse>>,
    notices: Option<Vec<Notice>>,
    links: Option<Vec<Link>>,
}

#[derive(Deserialize, Debug)]
struct HostSearchResponse {
    #[serde(rename = "nameserverSearchResults")]
    results: Option<Vec<Nameserver>>,
    notices: Option<Vec<Notice>>,
    links: Option<Vec<Link>>,
}

// ── formatting helpers ────────────────────────────────────────────────────────

fn label(s: &str) -> String {
    format!("\x1b[1;36m{:<18}\x1b[0m", s)
}

fn heading(s: &str) {
    println!("\n\x1b[1;33m{}\x1b[0m", s);
    println!("{}", "─".repeat(50));
}

fn row(key: &str, value: &str) {
    println!("  {}  {}", label(key), value);
}

fn print_events(events: &[Event]) {
    for e in events {
        let date = e.date.split('T').next().unwrap_or(&e.date);
        row(&e.action, date);
    }
}

fn print_entities(entities: &[Entity]) {
    for e in entities {
        let roles = e.roles.as_deref().unwrap_or_default().join(", ");
        let handle = e.handle.as_deref().unwrap_or("—");
        let name = vcard_field(&e.vcard_array, "fn").unwrap_or_default();
        let email = vcard_field(&e.vcard_array, "email").unwrap_or_default();

        let mut parts = vec![handle.to_string()];
        if !name.is_empty() {
            parts.push(name);
        }
        if !email.is_empty() {
            parts.push(format!("<{}>", email));
        }
        row(&roles, &parts.join("  "));
    }
}

fn print_next_link(links: &[Link]) {
    if let Some(next) = links.iter().find(|l| l.rel.as_deref() == Some("next")) {
        if let Some(href) = &next.href {
            println!("\n  \x1b[2mNext:  {}\x1b[0m", href);
        }
    }
}

fn print_notices(notices: &[Notice]) {
    for notice in notices {
        if let Some(title) = &notice.title {
            println!("\n  \x1b[2m{}\x1b[0m", title);
        }
        if let Some(lines) = &notice.description {
            for line in lines {
                println!("  \x1b[2m{}\x1b[0m", line);
            }
        }
    }
}

fn vcard_field(vcard: &Option<Value>, field: &str) -> Option<String> {
    let arr = vcard.as_ref()?.as_array()?;
    // vcardArray is ["vcard", [[type, params, kind, value], ...]]
    let entries = arr.get(1)?.as_array()?;
    for entry in entries {
        let e = entry.as_array()?;
        if e.first()?.as_str()? == field {
            return Some(e.get(3)?.as_str().unwrap_or("").to_string());
        }
    }
    None
}

// ── HTTP / options ────────────────────────────────────────────────────────────

type Auth<'a> = Option<(&'a str, &'a str)>;

struct Opts<'a> {
    auth: Auth<'a>,
    cursor: Option<&'a str>,
    count: Option<u32>,
    sort: Option<&'a str>,
    fields: Option<&'a str>,
    debug: bool,
}

async fn fetch<T: serde::de::DeserializeOwned>(
    client: &reqwest::Client,
    url: &str,
    opts: &Opts<'_>,
    extra: &[(&str, &str)],
) -> anyhow::Result<T> {
    let mut req = client.get(url);
    if let Some((user, pass)) = opts.auth {
        req = req.basic_auth(user, Some(pass));
    }
    let count_str = opts.count.map(|n| n.to_string());
    let mut params: Vec<(&str, &str)> = extra.to_vec();
    if let Some(c) = opts.cursor {
        params.push(("cursor", c));
    }
    if let Some(ref s) = count_str {
        params.push(("count", s));
    }
    if let Some(s) = opts.sort {
        params.push(("sort", s));
    }
    if let Some(f) = opts.fields {
        params.push(("fields", f));
    }
    if !params.is_empty() {
        req = req.query(&params);
    }
    let req = req.build()?;
    if opts.debug {
        eprintln!("\x1b[2m> {}\x1b[0m", req.url());
    }
    Ok(client.execute(req).await?.error_for_status()?.json().await?)
}

// ── lookup functions ──────────────────────────────────────────────────────────

async fn lookup_domain(
    client: &reqwest::Client,
    opts: &Opts<'_>,
    server: &str,
    name: &str,
) -> anyhow::Result<()> {
    let url = format!("{}/domain/{}", server, name);
    let resp: DomainResponse = fetch(client, &url, opts, &[]).await?;

    heading(&format!("Domain: {}", name.to_uppercase()));

    if let Some(h) = &resp.handle {
        row("Handle", h);
    }
    if let Some(n) = resp.unicode_name.as_ref().or(resp.ldh_name.as_ref()) {
        row("Name", n);
    }
    if let Some(statuses) = &resp.status {
        row("Status", &statuses.join(", "));
    }
    if let Some(ns) = &resp.nameservers {
        let names: Vec<&str> = ns.iter().filter_map(|n| n.ldh_name.as_deref()).collect();
        row("Nameservers", &names.join(", "));
    }
    if let Some(events) = &resp.events {
        println!();
        print_events(events);
    }
    if let Some(entities) = &resp.entities {
        println!();
        row("Contacts", "");
        print_entities(entities);
    }

    println!();
    Ok(())
}

async fn lookup_host(
    client: &reqwest::Client,
    opts: &Opts<'_>,
    server: &str,
    address: &str,
) -> anyhow::Result<()> {
    let url = format!("{}/ip/{}", server, address);
    let resp: HostResponse = fetch(client, &url, opts, &[]).await?;

    heading(&format!("Host / Network: {}", address));

    if let Some(h) = &resp.handle {
        row("Handle", h);
    }
    if let Some(n) = &resp.name {
        row("Name", n);
    }
    if let (Some(start), Some(end)) = (&resp.start_address, &resp.end_address) {
        row("Range", &format!("{} – {}", start, end));
    }
    if let Some(t) = &resp.ip_type {
        row("Type", t);
    }
    if let Some(c) = &resp.country {
        row("Country", c);
    }
    if let Some(statuses) = &resp.status {
        row("Status", &statuses.join(", "));
    }
    if let Some(events) = &resp.events {
        println!();
        print_events(events);
    }
    if let Some(entities) = &resp.entities {
        println!();
        row("Contacts", "");
        print_entities(entities);
    }

    println!();
    Ok(())
}

async fn lookup_entity(
    client: &reqwest::Client,
    opts: &Opts<'_>,
    server: &str,
    handle: &str,
) -> anyhow::Result<()> {
    let url = format!("{}/entity/{}", server, handle);
    let resp: EntityResponse = fetch(client, &url, opts, &[]).await?;

    heading(&format!("Entity: {}", handle));

    if let Some(h) = &resp.handle {
        row("Handle", h);
    }
    if let Some(c) = &resp.class {
        row("Class", c);
    }
    if let Some(roles) = &resp.roles {
        row("Roles", &roles.join(", "));
    }
    if let Some(name) = vcard_field(&resp.vcard_array, "fn") {
        row("Name", &name);
    }
    if let Some(org) = vcard_field(&resp.vcard_array, "org") {
        row("Organisation", &org);
    }
    if let Some(email) = vcard_field(&resp.vcard_array, "email") {
        row("Email", &email);
    }
    if let Some(tel) = vcard_field(&resp.vcard_array, "tel") {
        row("Phone", &tel);
    }
    if let Some(statuses) = &resp.status {
        row("Status", &statuses.join(", "));
    }
    if let Some(events) = &resp.events {
        println!();
        print_events(events);
    }
    if let Some(entities) = &resp.entities {
        println!();
        row("Contacts", "");
        print_entities(entities);
    }

    println!();
    Ok(())
}

async fn lookup_help(
    client: &reqwest::Client,
    opts: &Opts<'_>,
    server: &str,
) -> anyhow::Result<()> {
    let url = format!("{}/help", server);
    let resp: HelpResponse = fetch(client, &url, opts, &[]).await?;

    heading(&format!("Help: {}", server));

    let sections = resp
        .notices
        .into_iter()
        .flatten()
        .chain(resp.remarks.into_iter().flatten());

    for notice in sections {
        if let Some(title) = &notice.title {
            println!("\n  \x1b[1m{}\x1b[0m", title);
        }
        if let Some(lines) = &notice.description {
            for line in lines {
                println!("  {}", line);
            }
        }
    }

    println!();
    Ok(())
}

// ── search functions ──────────────────────────────────────────────────────────

async fn search_domains(
    client: &reqwest::Client,
    opts: &Opts<'_>,
    server: &str,
    name: &str,
) -> anyhow::Result<()> {
    let url = format!("{}/domains", server);
    let resp: DomainSearchResponse = fetch(client, &url, opts, &[("name", name)]).await?;
    let DomainSearchResponse { results, notices, links } = resp;

    heading(&format!("Domains matching \"{}\"", name));

    let results = results.unwrap_or_default();
    if results.is_empty() {
        println!("  No results.");
    } else {
        for d in &results {
            let n = d
                .unicode_name
                .as_ref()
                .or(d.ldh_name.as_ref())
                .map(|s| s.as_str())
                .unwrap_or("—");
            let status = d.status.as_deref().unwrap_or_default().join(", ");
            println!("  {:<42}  {}", n.to_uppercase(), status);
        }
    }

    if let Some(links) = links.as_deref() {
        print_next_link(links);
    }
    if let Some(notices) = notices.as_deref() {
        print_notices(notices);
    }
    println!();
    Ok(())
}

async fn search_entities(
    client: &reqwest::Client,
    opts: &Opts<'_>,
    server: &str,
    handle: Option<&str>,
    fn_name: Option<&str>,
) -> anyhow::Result<()> {
    let url = format!("{}/entities", server);
    let extra: Vec<(&str, &str)> = match (handle, fn_name) {
        (Some(h), _) => vec![("handle", h)],
        (_, Some(f)) => vec![("fn", f)],
        _ => anyhow::bail!("entities search requires --handle or --fn"),
    };
    let resp: EntitySearchResponse = fetch(client, &url, opts, &extra).await?;
    let EntitySearchResponse { results, notices, links } = resp;

    let pattern = handle.or(fn_name).unwrap_or("*");
    heading(&format!("Entities matching \"{}\"", pattern));

    let results = results.unwrap_or_default();
    if results.is_empty() {
        println!("  No results.");
    } else {
        for e in &results {
            let h = e.handle.as_deref().unwrap_or("—");
            let roles = e.roles.as_deref().unwrap_or_default().join(", ");
            let name = vcard_field(&e.vcard_array, "fn").unwrap_or_default();
            println!("  {:<24}  {:<20}  {}", h, roles, name);
        }
    }

    if let Some(links) = links.as_deref() {
        print_next_link(links);
    }
    if let Some(notices) = notices.as_deref() {
        print_notices(notices);
    }
    println!();
    Ok(())
}

async fn search_hosts(
    client: &reqwest::Client,
    opts: &Opts<'_>,
    server: &str,
    name: Option<&str>,
    ip: Option<&str>,
) -> anyhow::Result<()> {
    let url = format!("{}/hosts", server);
    let extra: Vec<(&str, &str)> = match (name, ip) {
        (Some(n), _) => vec![("name", n)],
        (_, Some(i)) => vec![("ip", i)],
        _ => anyhow::bail!("hosts search requires --name or --ip"),
    };
    let resp: HostSearchResponse = fetch(client, &url, opts, &extra).await?;
    let HostSearchResponse { results, notices, links } = resp;

    let pattern = name.or(ip).unwrap_or("*");
    heading(&format!("Nameservers matching \"{}\"", pattern));

    let results = results.unwrap_or_default();
    if results.is_empty() {
        println!("  No results.");
    } else {
        for ns in &results {
            println!("  {}", ns.ldh_name.as_deref().unwrap_or("—"));
        }
    }

    if let Some(links) = links.as_deref() {
        print_next_link(links);
    }
    if let Some(notices) = notices.as_deref() {
        print_notices(notices);
    }
    println!();
    Ok(())
}

// ── entry point ───────────────────────────────────────────────────────────────

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
    let client = builder.build().expect("failed to build HTTP client");

    let opts = Opts {
        auth: cli.user.as_deref().zip(cli.password.as_deref()),
        cursor: cli.cursor.as_deref(),
        count: cli.count,
        sort: cli.sort.as_deref(),
        fields: cli.fields.as_deref(),
        debug: cli.debug,
    };
    let server = cli
        .server
        .as_deref()
        .unwrap_or("https://rdap.org")
        .trim_end_matches('/');

    let result = match &cli.command {
        Command::Domain { name } => lookup_domain(&client, &opts, server, name).await,
        Command::Host { address } => lookup_host(&client, &opts, server, address).await,
        Command::Entity { handle } => lookup_entity(&client, &opts, server, handle).await,
        Command::Domains { name } => search_domains(&client, &opts, server, name).await,
        Command::Entities { handle, fn_name } => {
            search_entities(&client, &opts, server, handle.as_deref(), fn_name.as_deref()).await
        }
        Command::Hosts { name, ip } => {
            search_hosts(&client, &opts, server, name.as_deref(), ip.as_deref()).await
        }
        Command::Help => lookup_help(&client, &opts, server).await,
    };

    if let Err(e) = result {
        eprintln!("\x1b[1;31merror:\x1b[0m {}", e);
        std::process::exit(1);
    }
}
