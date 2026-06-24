use crate::client::{fetch, Opts};
use crate::format::{heading, print_next_cursor, print_notices, print_paging_metadata};
use crate::types::{
    vcard_field, DomainSearchResponse, EntitySearchResponse, HostSearchResponse,
};

pub async fn search_domains(
    client: &reqwest::Client,
    opts: &Opts<'_>,
    server: &str,
    name: &str,
) -> anyhow::Result<()> {
    let url = format!("{}/domains", server);
    let resp: DomainSearchResponse = fetch(client, &url, opts, &[("name", name)]).await?;
    let DomainSearchResponse { results, paging_metadata, notices, links } = resp;

    let nc = opts.no_color;
    heading(&format!("Domains matching \"{}\"", name), nc);

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

    if let Some(pm) = &paging_metadata {
        let next_links = pm.links.as_deref().filter(|l| !l.is_empty()).or(links.as_deref());
        print_paging_metadata(pm, next_links, nc);
    } else if let Some(links) = links.as_deref() {
        print_next_cursor(links, nc);
    }
    if let Some(notices) = notices.as_deref() {
        print_notices(notices, nc);
    }
    println!();
    Ok(())
}

pub async fn search_entities(
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
    let EntitySearchResponse { results, paging_metadata, notices, links } = resp;

    let nc = opts.no_color;
    let pattern = handle.or(fn_name).unwrap_or("*");
    heading(&format!("Entities matching \"{}\"", pattern), nc);

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

    if let Some(pm) = &paging_metadata {
        let next_links = pm.links.as_deref().filter(|l| !l.is_empty()).or(links.as_deref());
        print_paging_metadata(pm, next_links, nc);
    } else if let Some(links) = links.as_deref() {
        print_next_cursor(links, nc);
    }
    if let Some(notices) = notices.as_deref() {
        print_notices(notices, nc);
    }
    println!();
    Ok(())
}

pub async fn search_hosts(
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
    let HostSearchResponse { results, paging_metadata, notices, links } = resp;

    let nc = opts.no_color;
    let pattern = name.or(ip).unwrap_or("*");
    heading(&format!("Nameservers matching \"{}\"", pattern), nc);

    let results = results.unwrap_or_default();
    if results.is_empty() {
        println!("  No results.");
    } else {
        for ns in &results {
            println!("  {}", ns.ldh_name.as_deref().unwrap_or("—"));
        }
    }

    if let Some(pm) = &paging_metadata {
        let next_links = pm.links.as_deref().filter(|l| !l.is_empty()).or(links.as_deref());
        print_paging_metadata(pm, next_links, nc);
    } else if let Some(links) = links.as_deref() {
        print_next_cursor(links, nc);
    }
    if let Some(notices) = notices.as_deref() {
        print_notices(notices, nc);
    }
    println!();
    Ok(())
}
