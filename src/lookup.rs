use crate::client::Client;
use crate::types::{DomainResponse, EntityResponse, HelpResponse, NameserverResponse, NoridDomainCountResponse};

impl Client {
    pub async fn lookup_domain(&self, name: &str) -> anyhow::Result<()> {
        let url = format!("{}/domain/{name}", self.server);
        let resp: DomainResponse = self.fetch(&url, &[]).await?;
        let uname = name.to_uppercase();
        self.fmt.heading(&format!("Domain: {uname}"));
        resp.print(&self.fmt);
        println!();
        Ok(())
    }

    pub async fn lookup_nameserver(&self, hostname: &str) -> anyhow::Result<()> {
        let url = format!("{}/nameserver/{hostname}", self.server);
        let resp: NameserverResponse = self.fetch(&url, &[]).await?;
        self.fmt.heading(&format!("Nameserver: {hostname}"));
        resp.print(&self.fmt);
        println!();
        Ok(())
    }

    pub async fn lookup_entity(&self, handle: &str) -> anyhow::Result<()> {
        let url = format!("{}/entity/{handle}", self.server);
        let resp: EntityResponse = self.fetch(&url, &[]).await?;
        self.fmt.heading(&format!("Entity: {handle}"));
        resp.print(&self.fmt);
        println!();
        Ok(())
    }

    pub async fn lookup_help(&self) -> anyhow::Result<()> {
        let url = format!("{}/help", self.server);
        let resp: HelpResponse = self.fetch(&url, &[]).await?;
        let server = &self.server;
        self.fmt.heading(&format!("Help: {server}"));
        resp.print(&self.fmt);
        println!();
        Ok(())
    }

    pub async fn lookup_nameserver_handle(&self, handle: &str) -> anyhow::Result<()> {
        let url = format!("{}/nameserver_handle/{handle}", self.server);
        let resp: NameserverResponse = self.fetch(&url, &[]).await?;
        self.fmt.heading(&format!("Nameserver: {handle}"));
        resp.print(&self.fmt);
        println!();
        Ok(())
    }

    pub async fn lookup_norid_domain_count(&self, identity: &str) -> anyhow::Result<()> {
        let url = format!("{}/norid_domain_count/{identity}", self.server);
        let resp: NoridDomainCountResponse = self.fetch(&url, &[]).await?;
        self.fmt.heading(&format!("Norid Domain Count: {identity}"));
        resp.print(&self.fmt);
        println!();
        Ok(())
    }
}
