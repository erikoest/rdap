use clap::{Parser, Subcommand};
use rdap::client::{Client, ClientConfig};

#[derive(Parser)]
#[command(name = "rdap", about = "RDAP lookup client", disable_help_subcommand = true)]
#[allow(clippy::struct_excessive_bools)]
struct Cli {
    /// RDAP server base URL (default: <https://rdap.org>)
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
    /// Look up a nameserver by Norid handle [A-Z0-9-] (Norid extension)
    NameserverHandle { handle: String },
    /// Fetch the count of .no domains for a given identity [INPR0-9.] (Norid extension)
    NoridDomainCount { identity: String },
}

#[derive(Default)]
struct FileConfig {
    server: Option<String>,
    ipv4: bool,
    ipv6: bool,
    user: Option<String>,
    password: Option<String>,
    cursor: Option<String>,
    count: bool,
    sort: Option<String>,
    fields: Option<String>,
    debug: bool,
    no_color: bool,
}

fn load_config() -> FileConfig {
    let mut cfg = FileConfig::default();
    let home = match std::env::var_os("HOME") {
        Some(h) => h,
        None => return cfg,
    };
    let path = std::path::Path::new(&home).join(".rdap.conf");
    let contents = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return cfg,
    };
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, val)) = line.split_once('=') {
            match key.trim() {
                "server"   => cfg.server   = Some(val.trim().to_string()),
                "ipv4"     => cfg.ipv4     = val.trim() == "true",
                "ipv6"     => cfg.ipv6     = val.trim() == "true",
                "user"     => cfg.user     = Some(val.trim().to_string()),
                "password" => cfg.password = Some(val.trim().to_string()),
                "cursor"   => cfg.cursor   = Some(val.trim().to_string()),
                "count"    => cfg.count    = val.trim() == "true",
                "sort"     => cfg.sort     = Some(val.trim().to_string()),
                "fields"   => cfg.fields   = Some(val.trim().to_string()),
                "debug"    => cfg.debug    = val.trim() == "true",
                "no_color" | "no-color" => cfg.no_color = val.trim() == "true",
                _ => {}
            }
        }
    }
    cfg
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let file = load_config();

    // CLI flags override config file; for conflicting ipv4/ipv6, CLI wins if either is set.
    let (use_ipv4, use_ipv6) = if cli.ipv4 || cli.ipv6 {
        (cli.ipv4, cli.ipv6)
    } else {
        (file.ipv4, file.ipv6)
    };

    let local_addr: Option<std::net::IpAddr> = if use_ipv4 {
        Some(std::net::Ipv4Addr::UNSPECIFIED.into())
    } else if use_ipv6 {
        Some(std::net::Ipv6Addr::UNSPECIFIED.into())
    } else {
        None
    };

    let mut builder = reqwest::Client::builder().user_agent("rdap-cli/0.1");
    if let Some(addr) = local_addr {
        builder = builder.local_address(addr);
    }
    let http = builder.build().expect("failed to build HTTP client");

    let server = cli.server
        .or(file.server)
        .unwrap_or_else(|| "https://rdap.org".to_string());

    let auth = cli.user.zip(cli.password).or_else(|| file.user.zip(file.password));

    let client = Client::new(http, ClientConfig {
        server: server.trim_end_matches('/').to_string(),
        auth,
        cursor:   cli.cursor.or(file.cursor),
        count:    cli.count  || file.count,
        sort:     cli.sort.or(file.sort),
        fields:   cli.fields.or(file.fields),
        debug:    cli.debug  || file.debug,
        no_color: cli.no_color || file.no_color,
    });

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
        Command::NameserverHandle { handle } => client.lookup_nameserver_handle(handle).await,
        Command::NoridDomainCount { identity } => client.lookup_norid_domain_count(identity).await,
    };

    if let Err(e) = result {
        let esc = if cli.no_color || file.no_color { "\x1b[1m" } else { "\x1b[1;31m" };
        eprintln!("{esc}error:\x1b[0m {e}");
        std::process::exit(1);
    }
}
