use clap::{Parser, Subcommand};
use rdap::client::Opts;
use rdap::lookup::{lookup_domain, lookup_entity, lookup_help, lookup_host};
use rdap::search::{search_domains, search_entities, search_hosts};

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
    let client = builder.build().expect("failed to build HTTP client");

    let opts = Opts {
        auth: cli.user.as_deref().zip(cli.password.as_deref()),
        cursor: cli.cursor.as_deref(),
        count: cli.count,
        sort: cli.sort.as_deref(),
        fields: cli.fields.as_deref(),
        debug: cli.debug,
        no_color: cli.no_color,
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
        let esc = if cli.no_color { "\x1b[1m" } else { "\x1b[1;31m" };
        eprintln!("{}error:\x1b[0m {}", esc, e);
        std::process::exit(1);
    }
}
