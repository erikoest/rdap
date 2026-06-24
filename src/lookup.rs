use crate::client::{fetch, Opts};
use crate::format::{heading, print_entities, print_events, print_notices, row};
use crate::types::{
    vcard_field, DomainResponse, EntityResponse, HelpResponse, HostResponse, Notice,
};

pub async fn lookup_domain(
    client: &reqwest::Client,
    opts: &Opts<'_>,
    server: &str,
    name: &str,
) -> anyhow::Result<()> {
    let url = format!("{}/domain/{}", server, name);
    let resp: DomainResponse = fetch(client, &url, opts, &[]).await?;

    let nc = opts.no_color;
    heading(&format!("Domain: {}", name.to_uppercase()), nc);

    if let Some(h) = &resp.handle {
        row("Handle", h, nc);
    }
    if let Some(n) = resp.unicode_name.as_ref().or(resp.ldh_name.as_ref()) {
        row("Name", n, nc);
    }
    if let Some(statuses) = &resp.status {
        row("Status", &statuses.join(", "), nc);
    }
    if let Some(ns) = &resp.nameservers {
        let names: Vec<&str> = ns.iter().filter_map(|n| n.ldh_name.as_deref()).collect();
        row("Nameservers", &names.join(", "), nc);
    }
    if let Some(events) = &resp.events {
        println!();
        print_events(events, nc);
    }
    if let Some(entities) = &resp.entities {
        println!();
        row("Contacts", "", nc);
        print_entities(entities, nc);
    }

    println!();
    Ok(())
}

pub async fn lookup_host(
    client: &reqwest::Client,
    opts: &Opts<'_>,
    server: &str,
    address: &str,
) -> anyhow::Result<()> {
    let url = format!("{}/ip/{}", server, address);
    let resp: HostResponse = fetch(client, &url, opts, &[]).await?;

    let nc = opts.no_color;
    heading(&format!("Host / Network: {}", address), nc);

    if let Some(h) = &resp.handle {
        row("Handle", h, nc);
    }
    if let Some(n) = &resp.name {
        row("Name", n, nc);
    }
    if let (Some(start), Some(end)) = (&resp.start_address, &resp.end_address) {
        row("Range", &format!("{} – {}", start, end), nc);
    }
    if let Some(t) = &resp.ip_type {
        row("Type", t, nc);
    }
    if let Some(c) = &resp.country {
        row("Country", c, nc);
    }
    if let Some(statuses) = &resp.status {
        row("Status", &statuses.join(", "), nc);
    }
    if let Some(events) = &resp.events {
        println!();
        print_events(events, nc);
    }
    if let Some(entities) = &resp.entities {
        println!();
        row("Contacts", "", nc);
        print_entities(entities, nc);
    }

    println!();
    Ok(())
}

pub async fn lookup_entity(
    client: &reqwest::Client,
    opts: &Opts<'_>,
    server: &str,
    handle: &str,
) -> anyhow::Result<()> {
    let url = format!("{}/entity/{}", server, handle);
    let resp: EntityResponse = fetch(client, &url, opts, &[]).await?;

    let nc = opts.no_color;
    heading(&format!("Entity: {}", handle), nc);

    if let Some(h) = &resp.handle {
        row("Handle", h, nc);
    }
    if let Some(c) = &resp.class {
        row("Class", c, nc);
    }
    if let Some(roles) = &resp.roles {
        row("Roles", &roles.join(", "), nc);
    }
    if let Some(name) = vcard_field(&resp.vcard_array, "fn") {
        row("Name", &name, nc);
    }
    if let Some(org) = vcard_field(&resp.vcard_array, "org") {
        row("Organisation", &org, nc);
    }
    if let Some(email) = vcard_field(&resp.vcard_array, "email") {
        row("Email", &email, nc);
    }
    if let Some(tel) = vcard_field(&resp.vcard_array, "tel") {
        row("Phone", &tel, nc);
    }
    if let Some(statuses) = &resp.status {
        row("Status", &statuses.join(", "), nc);
    }
    if let Some(events) = &resp.events {
        println!();
        print_events(events, nc);
    }
    if let Some(entities) = &resp.entities {
        println!();
        row("Contacts", "", nc);
        print_entities(entities, nc);
    }

    println!();
    Ok(())
}

pub async fn lookup_help(
    client: &reqwest::Client,
    opts: &Opts<'_>,
    server: &str,
) -> anyhow::Result<()> {
    let url = format!("{}/help", server);
    let resp: HelpResponse = fetch(client, &url, opts, &[]).await?;

    let nc = opts.no_color;
    heading(&format!("Help: {}", server), nc);

    let sections: Vec<Notice> = resp
        .notices
        .into_iter()
        .flatten()
        .chain(resp.remarks.into_iter().flatten())
        .collect();
    print_notices(&sections, nc);

    println!();
    Ok(())
}
