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
        Command::NameserverHandle { handle } => client.lookup_nameserver_handle(handle).await,
        Command::NoridDomainCount { identity } => client.lookup_norid_domain_count(identity).await,
    };

    if let Err(e) = result {
        let esc = if cli.no_color { "\x1b[1m" } else { "\x1b[1;31m" };
        eprintln!("{}error:\x1b[0m {}", esc, e);
        std::process::exit(1);
    }
}
