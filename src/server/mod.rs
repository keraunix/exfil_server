use anyhow::Context;
use axum::{
    Router,
    body::Bytes,
    extract::ConnectInfo,
    http::{HeaderMap, Method, Uri},
    routing::any,
};
use std::borrow::Cow;
use std::env;
use std::net::SocketAddr;

pub async fn init_server() -> anyhow::Result<()> {
    let config = build_config()?;
    let app = build_app().into_make_service_with_connect_info::<SocketAddr>();

    let addr: SocketAddr = format!("0.0.0.0:{}", config.port)
        .parse()
        .context("invalid bind address")?;

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind {}", addr))?;
    tracing::debug!("server starting at: {:?}", addr);
    axum::serve(listener, app).await.context("server error")?;

    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
struct Config {
    port: String,
}

fn build_config() -> anyhow::Result<Config> {
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    let config = Config { port };
    Ok(config)
}

fn build_app() -> Router {
    Router::new()
        .route("/", any(log_handler))
        .route("/{*wildcard}", any(log_handler))
}

async fn log_handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> &'static str {
    const MAX: usize = 64 * 1024;
    let path = uri
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or_else(|| uri.path());

    let addr_str = addr.to_string();
    let summary = summarize_request(method.as_str(), path, &addr_str, &headers, &body, MAX);

    tracing::info!(
        event = "http.request",
        method = %summary.method,
        path = %summary.path,
        ip = %summary.ip,
        headers = %summary.headers,
        body = %summary.body_preview,
        body_truncated = summary.truncated,
        "received"
    );

    "logged request"
}

#[derive(Debug, PartialEq, Eq)]
struct ReqSummary<'a> {
    method: &'a str,
    path: &'a str,
    ip: &'a str,
    headers: String,
    body_preview: Cow<'a, str>,
    truncated: bool,
}

fn summarize_request<'a>(
    method: &'a str,
    path: &'a str,
    ip: &'a str,
    headers: &HeaderMap,
    body: &'a [u8],
    max: usize,
) -> ReqSummary<'a> {
    let headers = headers_to_string(headers);
    let body_preview = String::from_utf8_lossy(&body[..body.len().min(max)]);
    let truncated = body.len() > max;
    ReqSummary {
        method,
        path,
        ip,
        headers,
        body_preview,
        truncated,
    }
}

fn headers_to_string(h: &HeaderMap) -> String {
    h.iter()
        .map(|(k, v)| {
            let key = k.as_str();
            let val = v.to_str().unwrap_or("<non-utf8>");
            format!("{}: {}", key, val)
        })
        .collect::<Vec<String>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderMap, HeaderValue};

    // ---------- Unit tests (pure helpers) ----------

    #[test]
    fn headers_to_string_includes_all_values_and_duplicates() {
        let mut h = HeaderMap::new();
        h.insert("Host", HeaderValue::from_static("localhost:8080"));
        h.insert("User-Agent", HeaderValue::from_static("curl/8.5.0"));
        // duplicate header
        h.append("Accept", HeaderValue::from_static("*/*"));
        h.append("Accept", HeaderValue::from_static("text/plain"));

        let s = headers_to_string(&h);
        let s = s.to_ascii_lowercase();

        // Don’t depend on ordering; just check substrings are present.
        assert!(s.contains("host: localhost:8080"));
        assert!(s.contains("user-agent: curl/8.5.0"));
        assert!(s.contains("accept: */*"));
        assert!(s.contains("accept: text/plain"));
    }

    #[test]
    fn summarize_request_captures_method_path_ip_and_no_truncation() {
        let mut h = HeaderMap::new();
        h.insert("Content-Type", HeaderValue::from_static("text/plain"));

        let body = b"hello world";
        let s = summarize_request("POST", "/tree?x=1", "127.0.0.1:12345", &h, body, 64);

        assert_eq!(s.method, "POST");
        assert_eq!(s.path, "/tree?x=1"); // full path+query as passed in
        assert_eq!(s.ip, "127.0.0.1:12345");
        assert!(
            s.headers
                .to_ascii_lowercase()
                .contains("content-type: text/plain")
        );
        assert_eq!(s.body_preview, "hello world");
        assert_eq!(s.truncated, false);
    }

    #[test]
    fn summarize_request_truncates_body_at_max() {
        let h = HeaderMap::new();
        let body = b"abcdefghijklmnopqrstuvwxyz";
        let s = summarize_request("PUT", "/big", "10.0.0.1:5555", &h, body, 10);
        assert_eq!(s.body_preview, "abcdefghij");
        assert!(s.truncated);
    }

    #[test]
    fn summarize_request_handles_non_utf8_body_lossily() {
        let h = HeaderMap::new();
        let body = b"\xff\xfeA\x80B"; // invalid UTF-8
        let s = summarize_request("POST", "/bin", "1.2.3.4:42", &h, body, 64);
        // “�” replacement chars should appear; length == original len
        assert_eq!(s.body_preview.chars().count(), 5);
        assert!(s.body_preview.contains('�'));
    }
}
