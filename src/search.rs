use crate::client::Client;
use crate::pager::{PageAction, prompt_next_page};
use crate::types::{DomainSearchResponse, EntitySearchResponse, NameserverSearchResponse, Link, PagingMetadata};

impl Client {
    pub async fn search_domains(&self, name: &str) -> anyhow::Result<()> {
        let url = format!("{}/domains", self.server);
        let resp: DomainSearchResponse = self.fetch(&url, &[("name", name)]).await?;
        self.fmt.heading(&format!("Domains matching \"{name}\""));
        resp.print(&self.fmt);
        println!();

        let mut next = next_href(&resp.paging_metadata, &resp.links).map(str::to_owned);
        while let Some(next_url) = next {
            match prompt_next_page(self.fmt.nc) {
                PageAction::Quit => break,
                PageAction::Next => {
                    let resp: DomainSearchResponse = self.fetch_url(&next_url).await?;
                    next = next_href(&resp.paging_metadata, &resp.links).map(str::to_owned);
                    resp.print(&self.fmt);
                    println!();
                }
            }
        }

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
        self.fmt.heading(&format!("Entities matching \"{pattern}\""));
        resp.print(&self.fmt);
        println!();

        let mut next = next_href(&resp.paging_metadata, &resp.links).map(str::to_owned);
        while let Some(next_url) = next {
            match prompt_next_page(self.fmt.nc) {
                PageAction::Quit => break,
                PageAction::Next => {
                    let resp: EntitySearchResponse = self.fetch_url(&next_url).await?;
                    next = next_href(&resp.paging_metadata, &resp.links).map(str::to_owned);
                    resp.print(&self.fmt);
                    println!();
                }
            }
        }

        Ok(())
    }

    pub async fn search_nameservers(
        &self,
        name: Option<&str>,
        ip: Option<&str>,
    ) -> anyhow::Result<()> {
        let url = format!("{}/nameservers", self.server);
        let extra: Vec<(&str, &str)> = match (name, ip) {
            (Some(n), _) => vec![("name", n)],
            (_, Some(i)) => vec![("ip", i)],
            _ => anyhow::bail!("nameservers search requires --name or --ip"),
        };
        let resp: NameserverSearchResponse = self.fetch(&url, &extra).await?;
        let pattern = name.or(ip).unwrap_or("*");
        self.fmt.heading(&format!("Nameservers matching \"{pattern}\""));
        resp.print(&self.fmt);
        println!();

        let mut next = next_href(&resp.paging_metadata, &resp.links).map(str::to_owned);
        while let Some(next_url) = next {
            match prompt_next_page(self.fmt.nc) {
                PageAction::Quit => break,
                PageAction::Next => {
                    let resp: NameserverSearchResponse = self.fetch_url(&next_url).await?;
                    next = next_href(&resp.paging_metadata, &resp.links).map(str::to_owned);
                    resp.print(&self.fmt);
                    println!();
                }
            }
        }

        Ok(())
    }
}

fn next_href<'a>(pm: &'a Option<PagingMetadata>, links: &'a Option<Vec<Link>>) -> Option<&'a str> {
    pm.as_ref()
        .and_then(|p| p.next_href())
        .or_else(|| {
            links.as_deref()?
                .iter()
                .find(|l| l.rel.as_deref() == Some("next"))?
                .href.as_deref()
        })
}
