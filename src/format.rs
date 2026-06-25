use crate::types::{
    DomainResponse, DomainSearchResponse, Entity, EntityResponse, EntitySearchResponse,
    NoridDomainCountResponse,
    Event, HelpResponse, HostResponse, HostSearchResponse, Link, Notice, PagingMetadata,
    vcard_field,
};

pub const RULE: &str = "──────────────────────────────────────────────────";

// ── Formatter ────────────────────────────────────────────────────────────────

pub struct Formatter {
    pub(crate) nc: bool,
}

impl Formatter {
    #[must_use]
    pub fn new(nc: bool) -> Self {
        Self { nc }
    }

    pub(crate) fn dim(&self) -> &'static str {
        if self.nc { "\x1b[3m" } else { "\x1b[2m" }
    }

    fn label(&self, s: &str) -> String {
        let esc = if self.nc { "\x1b[1m" } else { "\x1b[1;36m" };
        format!("{esc}{s:<18}\x1b[0m")
    }

    pub fn heading(&self, s: &str) {
        let esc = if self.nc { "\x1b[1;4m" } else { "\x1b[1;33m" };
        println!("\n{esc}{s}\x1b[0m");
        println!("{RULE}");
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
            if !email.is_empty() { parts.push(format!("<{email}>")); }
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
            (Some(n), Some(s)) => self.row("Page", &format!("{n} (size {s})")),
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

#[must_use]
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
            for n in ns {
                let name = n.ldh_name.as_deref().unwrap_or("—");
                match n.handle.as_deref() {
                    Some(h) => fmt.row("Nameserver", &format!("{name}  {h}")),
                    None    => fmt.row("Nameserver", name),
                }
            }
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
            fmt.row("Range", &format!("{start} – {end}"));
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
        if let Some(ids) = &self.public_ids {
            for id in ids {
                let label = id.id_type.as_deref().unwrap_or("ID");
                let value = id.identifier.as_deref().unwrap_or("—");
                fmt.row(label, value);
            }
        }
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
                    .map_or("—", String::as_str);
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
                println!("  {h:<24}  {roles:<20}  {name}");
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

impl NoridDomainCountResponse {
    pub fn print(&self, fmt: &Formatter) {
        if let Some(entries) = &self.domain_count {
            for dc in entries {
                let domain = dc.parent_domain_name.as_deref().unwrap_or("—");
                let count  = dc.count.map_or_else(|| "—".to_string(), |n| n.to_string());
                fmt.row(domain, &count);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_cursor_finds_cursor_param() {
        let href = "https://rdap.org/domains?name=example*&cursor=page2token";
        assert_eq!(extract_cursor(href), Some("page2token".to_string()));
    }

    #[test]
    fn extract_cursor_percent_decodes_value() {
        let href = "https://rdap.org/domains?name=ex*&cursor=key%3Dvalue%26more";
        assert_eq!(extract_cursor(href), Some("key=value&more".to_string()));
    }

    #[test]
    fn extract_cursor_returns_none_when_param_absent() {
        let href = "https://rdap.org/domains?name=example*";
        assert_eq!(extract_cursor(href), None);
    }

    #[test]
    fn extract_cursor_returns_none_for_invalid_url() {
        assert_eq!(extract_cursor("not a url"), None);
    }

    #[test]
    fn extract_cursor_ignores_other_params() {
        let href = "https://rdap.org/domains?name=ex*&sort=name&cursor=tok&fields=ldhName";
        assert_eq!(extract_cursor(href), Some("tok".to_string()));
    }
}
