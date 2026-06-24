use crate::format::dim;

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
    if let Some((user, pass)) = opts.auth {
        req = req.basic_auth(user, Some(pass));
    }
    let mut params: Vec<(&str, &str)> = extra.to_vec();
    if let Some(c) = opts.cursor {
        params.push(("cursor", c));
    }
    if opts.count {
        params.push(("count", "true"));
    }
    if let Some(s) = opts.sort {
        params.push(("sort", s));
    }
    if let Some(f) = opts.fields {
        params.push(("fields", f));
    }
    if !params.is_empty() {
        req = req.query(&params);
    }
    let req = req.build()?;
    if opts.debug {
        eprintln!("{}> {}\x1b[0m", dim(opts.no_color), req.url());
    }
    Ok(client.execute(req).await?.error_for_status()?.json().await?)
}
