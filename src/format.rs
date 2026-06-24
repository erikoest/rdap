use crate::types::{Entity, Event, Link, Notice, PagingMetadata, vcard_field};

pub const RULE: &str = "──────────────────────────────────────────────────";

pub fn dim(nc: bool) -> &'static str {
    if nc { "\x1b[3m" } else { "\x1b[2m" }
}

pub fn label(s: &str, nc: bool) -> String {
    let esc = if nc { "\x1b[1m" } else { "\x1b[1;36m" };
    format!("{}{:<18}\x1b[0m", esc, s)
}

pub fn heading(s: &str, nc: bool) {
    let esc = if nc { "\x1b[1;4m" } else { "\x1b[1;33m" };
    println!("\n{}{}\x1b[0m", esc, s);
    println!("{}", RULE);
}

pub fn row(key: &str, value: &str, nc: bool) {
    println!("  {}  {}", label(key, nc), value);
}

pub fn print_events(events: &[Event], nc: bool) {
    for e in events {
        let date = e.date.split('T').next().unwrap_or(&e.date);
        row(&e.action, date, nc);
    }
}

pub fn print_entities(entities: &[Entity], nc: bool) {
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
        row(&roles, &parts.join("  "), nc);
    }
}

pub fn extract_cursor(href: &str) -> Option<String> {
    url::Url::parse(href).ok()?
        .query_pairs()
        .find(|(k, _)| k == "cursor")
        .map(|(_, v)| v.into_owned())
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
    if let Some(count) = pm.total_count {
        row("Total", &count.to_string(), nc);
    }
    match (pm.page_number, pm.page_size) {
        (Some(n), Some(s)) => row("Page", &format!("{} (size {})", n, s), nc),
        (Some(n), None)    => row("Page", &n.to_string(), nc),
        (None, Some(s))    => row("Page size", &s.to_string(), nc),
        (None, None)       => {}
    }
    if let Some(links) = links {
        print_next_cursor(links, nc);
    }
}

pub fn print_notices(notices: &[Notice], nc: bool) {
    for notice in notices {
        if let Some(title) = &notice.title {
            println!("\n  {}{}\x1b[0m", dim(nc), title);
        }
        if let Some(lines) = &notice.description {
            for line in lines {
                println!("  {}{}\x1b[0m", dim(nc), line);
            }
        }
    }
}
