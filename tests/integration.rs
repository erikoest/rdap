use rdap::client::{Client, ClientConfig};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

// ── mock HTTP server ──────────────────────────────────────────────────────────

struct MockServer {
    addr: std::net::SocketAddr,
    requests: Arc<Mutex<Vec<String>>>,
    _handle: tokio::task::JoinHandle<()>,
}

impl MockServer {
    async fn new(body: &'static str) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let requests: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let reqs = Arc::clone(&requests);

        let handle = tokio::spawn(async move {
            loop {
                let Ok((mut stream, _)) = listener.accept().await else { return };
                let mut buf = vec![0u8; 8192];
                let mut total = 0;
                loop {
                    let n = stream.read(&mut buf[total..]).await.unwrap_or(0);
                    total += n;
                    if n == 0 || buf[..total].windows(4).any(|w| w == b"\r\n\r\n") { break; }
                }
                reqs.lock().await.push(String::from_utf8_lossy(&buf[..total]).to_string());
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/rdap+json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(), body
                );
                stream.write_all(resp.as_bytes()).await.ok();
            }
        });

        Self { addr, requests, _handle: handle }
    }

    fn url(&self) -> String {
        format!("http://{}", self.addr)
    }

    async fn last_request(&self) -> String {
        self.requests.lock().await.last().cloned().unwrap_or_default()
    }
}

fn cfg(server_url: &str) -> ClientConfig {
    ClientConfig {
        server: server_url.to_string(),
        auth: None, cursor: None, count: false,
        sort: None, fields: None, debug: false, no_color: true,
    }
}

fn client(server_url: &str) -> Client {
    Client::new(reqwest::Client::new(), cfg(server_url))
}

fn client_with_auth(server_url: &str, user: &str, pass: &str) -> Client {
    Client::new(reqwest::Client::new(), ClientConfig {
        auth: Some((user.to_string(), pass.to_string())),
        ..cfg(server_url)
    })
}

fn client_with_opts(server_url: &str, cursor: Option<&str>, count: bool,
    sort: Option<&str>, fields: Option<&str>) -> Client {
    Client::new(reqwest::Client::new(), ClientConfig {
        cursor: cursor.map(str::to_string),
        count,
        sort: sort.map(str::to_string),
        fields: fields.map(str::to_string),
        ..cfg(server_url)
    })
}

// ── RDAP response fixtures ────────────────────────────────────────────────────

const DOMAIN_JSON: &str = r#"{"ldhName":"example.com","handle":"D1234","status":["active"]}"#;
const HOST_JSON: &str = r#"{"handle":"NET-8-8-8-0","startAddress":"8.8.8.0","endAddress":"8.8.8.255"}"#;
const ENTITY_JSON: &str = r#"{"handle":"GOGL","objectClassName":"entity"}"#;
const HELP_JSON: &str = r#"{"notices":[{"title":"About","description":["RDAP test server"]}]}"#;
const DOMAIN_SEARCH_JSON: &str = r#"{"domainSearchResults":[{"ldhName":"example.com"}]}"#;
const ENTITY_SEARCH_JSON: &str = r#"{"entitySearchResults":[{"handle":"GOGL","objectClassName":"entity"}]}"#;
const HOST_SEARCH_JSON: &str = r#"{"nameserverSearchResults":[{"ldhName":"ns1.example.com"}]}"#;
const DOMAIN_SEARCH_PAGED_JSON: &str = r#"{
    "domainSearchResults": [{"ldhName": "example.com"}],
    "paging_metadata": {
        "totalCount": 100,
        "pageNumber": 1,
        "pageSize": 10,
        "links": [{"rel": "next", "href": "http://rdap.example/domains?name=ex*&cursor=tok2"}]
    }
}"#;

// ── URL / path routing ────────────────────────────────────────────────────────

#[tokio::test]
async fn lookup_domain_requests_correct_path() {
    let srv = MockServer::new(DOMAIN_JSON).await;
    client(&srv.url()).lookup_domain("example.com").await.unwrap();
    assert!(srv.last_request().await.starts_with("GET /domain/example.com HTTP/1.1"));
}

#[tokio::test]
async fn lookup_host_requests_correct_path() {
    let srv = MockServer::new(HOST_JSON).await;
    client(&srv.url()).lookup_host("8.8.8.8").await.unwrap();
    assert!(srv.last_request().await.starts_with("GET /ip/8.8.8.8 HTTP/1.1"));
}

#[tokio::test]
async fn lookup_entity_requests_correct_path() {
    let srv = MockServer::new(ENTITY_JSON).await;
    client(&srv.url()).lookup_entity("GOGL").await.unwrap();
    assert!(srv.last_request().await.starts_with("GET /entity/GOGL HTTP/1.1"));
}

#[tokio::test]
async fn lookup_help_requests_correct_path() {
    let srv = MockServer::new(HELP_JSON).await;
    client(&srv.url()).lookup_help().await.unwrap();
    assert!(srv.last_request().await.starts_with("GET /help HTTP/1.1"));
}

#[tokio::test]
async fn search_domains_sends_name_param() {
    let srv = MockServer::new(DOMAIN_SEARCH_JSON).await;
    client(&srv.url()).search_domains("example*").await.unwrap();
    let req = srv.last_request().await;
    assert!(req.starts_with("GET /domains?"), "path: {req}");
    assert!(req.contains("name=example"), "param: {req}");
}

#[tokio::test]
async fn search_entities_with_handle_sends_handle_param() {
    let srv = MockServer::new(ENTITY_SEARCH_JSON).await;
    client(&srv.url()).search_entities(Some("GOGL*"), None).await.unwrap();
    let req = srv.last_request().await;
    assert!(req.starts_with("GET /entities?"), "path: {req}");
    assert!(req.contains("handle=GOGL"), "param: {req}");
}

#[tokio::test]
async fn search_entities_with_fn_sends_fn_param() {
    let srv = MockServer::new(ENTITY_SEARCH_JSON).await;
    client(&srv.url()).search_entities(None, Some("John*")).await.unwrap();
    let req = srv.last_request().await;
    assert!(req.contains("fn=John"), "param: {req}");
}

#[tokio::test]
async fn search_hosts_with_name_sends_name_param() {
    let srv = MockServer::new(HOST_SEARCH_JSON).await;
    client(&srv.url()).search_hosts(Some("ns1*"), None).await.unwrap();
    let req = srv.last_request().await;
    assert!(req.starts_with("GET /hosts?"), "path: {req}");
    assert!(req.contains("name=ns1"), "param: {req}");
}

#[tokio::test]
async fn search_hosts_with_ip_sends_ip_param() {
    let srv = MockServer::new(HOST_SEARCH_JSON).await;
    client(&srv.url()).search_hosts(None, Some("8.8.8.*")).await.unwrap();
    let req = srv.last_request().await;
    assert!(req.contains("ip="), "param: {req}");
}

// ── RFC-8977 query parameters ─────────────────────────────────────────────────

#[tokio::test]
async fn cursor_param_is_forwarded() {
    let srv = MockServer::new(DOMAIN_SEARCH_JSON).await;
    client_with_opts(&srv.url(), Some("page2token"), false, None, None)
        .search_domains("ex*").await.unwrap();
    assert!(srv.last_request().await.contains("cursor=page2token"));
}

#[tokio::test]
async fn count_true_sends_count_param() {
    let srv = MockServer::new(DOMAIN_SEARCH_JSON).await;
    client_with_opts(&srv.url(), None, true, None, None)
        .search_domains("ex*").await.unwrap();
    assert!(srv.last_request().await.contains("count=true"));
}

#[tokio::test]
async fn sort_param_is_forwarded() {
    let srv = MockServer::new(DOMAIN_SEARCH_JSON).await;
    client_with_opts(&srv.url(), None, false, Some("-name"), None)
        .search_domains("ex*").await.unwrap();
    assert!(srv.last_request().await.contains("sort="));
}

#[tokio::test]
async fn fields_param_is_forwarded() {
    let srv = MockServer::new(DOMAIN_JSON).await;
    client_with_opts(&srv.url(), None, false, None, Some("ldhName,handle"))
        .lookup_domain("example.com").await.unwrap();
    assert!(srv.last_request().await.contains("fields="));
}

// ── RFC-2617 basic auth ───────────────────────────────────────────────────────

#[tokio::test]
async fn basic_auth_sends_authorization_header() {
    let srv = MockServer::new(DOMAIN_JSON).await;
    client_with_auth(&srv.url(), "alice", "secret")
        .lookup_domain("example.com").await.unwrap();
    let req = srv.last_request().await;
    // reqwest lowercases header names (valid per HTTP spec)
    assert!(req.to_ascii_lowercase().contains("authorization: basic "), "got: {req}");
}

// ── paging metadata ───────────────────────────────────────────────────────────

#[tokio::test]
async fn paged_search_response_is_parsed_without_error() {
    let srv = MockServer::new(DOMAIN_SEARCH_PAGED_JSON).await;
    client(&srv.url()).search_domains("ex*").await.unwrap();
}

// ── validation errors (no HTTP needed) ───────────────────────────────────────

#[tokio::test]
async fn search_entities_errors_without_handle_or_fn() {
    let c = client("http://localhost");
    assert!(c.search_entities(None, None).await.is_err());
}

#[tokio::test]
async fn search_hosts_errors_without_name_or_ip() {
    let c = client("http://localhost");
    assert!(c.search_hosts(None, None).await.is_err());
}
