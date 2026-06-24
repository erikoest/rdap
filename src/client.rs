use crate::format::Formatter;

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
