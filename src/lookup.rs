use crate::client::Client;
use crate::types::{DomainResponse, EntityResponse, HelpResponse, HostResponse};

impl Client {
    pub async fn lookup_domain(&self, name: &str) -> anyhow::Result<()> {
        let url = format!("{}/domain/{}", self.server, name);
        let resp: DomainResponse = self.fetch(&url, &[]).await?;
        self.fmt.heading(&format!("Domain: {}", name.to_uppercase()));
        resp.print(&self.fmt);
        println!();
        Ok(())
    }

    pub async fn lookup_host(&self, address: &str) -> anyhow::Result<()> {
        let url = format!("{}/ip/{}", self.server, address);
        let resp: HostResponse = self.fetch(&url, &[]).await?;
        self.fmt.heading(&format!("Host / Network: {}", address));
        resp.print(&self.fmt);
        println!();
        Ok(())
    }

    pub async fn lookup_entity(&self, handle: &str) -> anyhow::Result<()> {
        let url = format!("{}/entity/{}", self.server, handle);
        let resp: EntityResponse = self.fetch(&url, &[]).await?;
        self.fmt.heading(&format!("Entity: {}", handle));
        resp.print(&self.fmt);
        println!();
        Ok(())
    }

    pub async fn lookup_help(&self) -> anyhow::Result<()> {
        let url = format!("{}/help", self.server);
        let resp: HelpResponse = self.fetch(&url, &[]).await?;
        self.fmt.heading(&format!("Help: {}", self.server));
        resp.print(&self.fmt);
        println!();
        Ok(())
    }
}
