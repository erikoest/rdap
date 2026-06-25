use crate::format::Formatter;

pub struct ClientConfig {
    pub server: String,
    pub auth: Option<(String, String)>,
    pub cursor: Option<String>,
    pub count: bool,
    pub sort: Option<String>,
    pub fields: Option<String>,
    pub debug: bool,
    pub no_color: bool,
}

pub struct Client {
    http: reqwest::Client,
    pub(crate) server: String,
    auth: Option<(String, String)>,
    cursor: Option<String>,
    count: bool,
    sort: Option<String>,
    fields: Option<String>,
    debug: bool,
    pub(crate) fmt: Formatter,
}

impl Client {
    #[must_use]
    pub fn new(http: reqwest::Client, cfg: ClientConfig) -> Self {
        Self {
            http,
            server: cfg.server,
            auth: cfg.auth,
            cursor: cfg.cursor,
            count: cfg.count,
            sort: cfg.sort,
            fields: cfg.fields,
            debug: cfg.debug,
            fmt: Formatter::new(cfg.no_color),
        }
    }

    pub(crate) async fn fetch<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        extra: &[(&str, &str)],
    ) -> anyhow::Result<T> {
        let mut req = self.http.get(url);
        if let Some((user, pass)) = &self.auth {
            req = req.basic_auth(user, Some(pass.as_str()));
        }
        let mut params: Vec<(&str, &str)> = extra.to_vec();
        if let Some(c) = &self.cursor { params.push(("cursor", c)); }
        if self.count { params.push(("count", "true")); }
        if let Some(s) = &self.sort { params.push(("sort", s)); }
        if let Some(f) = &self.fields { params.push(("fields", f)); }
        if !params.is_empty() { req = req.query(&params); }
        let req = req.build()?;
        if self.debug {
            eprintln!("{}> {}\x1b[0m", self.fmt.dim(), req.url());
        }
        Ok(self.http.execute(req).await?.error_for_status()?.json().await?)
    }

    /// Fetch a complete URL verbatim — used for `rel=next` paging links that
    /// already carry their own query parameters.
    pub(crate) async fn fetch_url<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
    ) -> anyhow::Result<T> {
        let mut req = self.http.get(url);
        if let Some((user, pass)) = &self.auth {
            req = req.basic_auth(user, Some(pass.as_str()));
        }
        let req = req.build()?;
        if self.debug {
            eprintln!("{}> {}\x1b[0m", self.fmt.dim(), req.url());
        }
        Ok(self.http.execute(req).await?.error_for_status()?.json().await?)
    }
}
