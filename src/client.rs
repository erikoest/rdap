use crate::format::Formatter;

// ── Client ───────────────────────────────────────────────────────────────────

#[allow(dead_code)]
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

#[allow(dead_code)]
impl Client {
    pub fn new(
        http: reqwest::Client,
        server: String,
        auth: Option<(String, String)>,
        cursor: Option<String>,
        count: bool,
        sort: Option<String>,
        fields: Option<String>,
        debug: bool,
        no_color: bool,
    ) -> Self {
        Self { http, server, auth, cursor, count, sort, fields, debug, fmt: Formatter::new(no_color) }
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
}

// ── legacy Opts / fetch (kept for compilation; removed in Task 3) ─────────────

pub type Auth<'a> = Option<(&'a str, &'a str)>;

pub struct Opts<'a> {
    pub auth: Auth<'a>,
    pub cursor: Option<&'a str>,
    pub count: bool,
    pub sort: Option<&'a str>,
    pub fields: Option<&'a str>,
    pub debug: bool,
    pub no_color: bool,
}

pub async fn fetch<T: serde::de::DeserializeOwned>(
    client: &reqwest::Client,
    url: &str,
    opts: &Opts<'_>,
    extra: &[(&str, &str)],
) -> anyhow::Result<T> {
    let mut req = client.get(url);
    if let Some((user, pass)) = opts.auth { req = req.basic_auth(user, Some(pass)); }
    let mut params: Vec<(&str, &str)> = extra.to_vec();
    if let Some(c) = opts.cursor { params.push(("cursor", c)); }
    if opts.count { params.push(("count", "true")); }
    if let Some(s) = opts.sort { params.push(("sort", s)); }
    if let Some(f) = opts.fields { params.push(("fields", f)); }
    if !params.is_empty() { req = req.query(&params); }
    let req = req.build()?;
    if opts.debug {
        eprintln!("{}> {}\x1b[0m", dim(opts.no_color), req.url());
    }
    Ok(client.execute(req).await?.error_for_status()?.json().await?)
}

use crate::format::dim;
