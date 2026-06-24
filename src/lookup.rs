use crate::client::Client;
use crate::types::{DomainResponse, EntityResponse, HelpResponse, HostResponse, NoridDomainCountResponse};

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

    pub async fn lookup_nameserver_handle(&self, handle: &str) -> anyhow::Result<()> {
        let url = format!("{}/nameserver_handle/{}", self.server, handle);
        let resp: HostResponse = self.fetch(&url, &[]).await?;
        self.fmt.heading(&format!("Nameserver: {}", handle));
        resp.print(&self.fmt);
        println!();
        Ok(())
    }

    pub async fn lookup_norid_domain_count(&self, query: &str) -> anyhow::Result<()> {
        let url = format!("{}/norid_domain_count/{}", self.server, query);
        let resp: NoridDomainCountResponse = self.fetch(&url, &[]).await?;
        self.fmt.heading(&format!("Norid Domain Count: {}", query));
        resp.print(&self.fmt);
        println!();
        Ok(())
    }
}
