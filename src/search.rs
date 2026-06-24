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
