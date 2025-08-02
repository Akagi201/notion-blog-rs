use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, Method, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use moka::future::Cache;
use regex::Regex;
use reqwest::Client;
use tracing::{debug, error, info};

use crate::{
    config::{Config, DomainConfig},
    error::{ProxyError, Result},
    rewriter::HtmlRewriter,
};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub client: Client,
    pub cache: Cache<String, DomainConfig>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let client = Client::new();
        let cache = Cache::builder()
            .max_capacity(config.cache.max_capacity)
            .time_to_live(std::time::Duration::from_secs(
                config.cache.time_to_live_secs,
            ))
            .build();

        Self {
            config: Arc::new(config),
            client,
            cache,
        }
    }

    pub async fn get_domain_config(&self, hostname: &str) -> Result<DomainConfig> {
        // Remove www prefix
        let hostname = hostname.strip_prefix("www.").unwrap_or(hostname);

        // Check cache first
        if let Some(config) = self.cache.get(hostname).await {
            return Ok(config);
        }

        // Look up in config
        if let Some(mut domain_config) = self.config.domains.get(hostname).cloned() {
            domain_config.compute_derived_fields();
            self.cache
                .insert(hostname.to_string(), domain_config.clone())
                .await;
            Ok(domain_config)
        } else {
            Err(ProxyError::DomainNotFound(hostname.to_string()))
        }
    }
}

pub async fn proxy_handler(
    State(state): State<AppState>,
    uri: Uri,
    method: Method,
    headers: HeaderMap,
    body: Body,
) -> impl IntoResponse {
    match handle_request(state, uri, method, headers, body).await {
        Ok(response) => response,
        Err(e) => {
            error!("Request failed: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

async fn handle_request(
    state: AppState,
    uri: Uri,
    method: Method,
    headers: HeaderMap,
    body: Body,
) -> Result<Response> {
    let original_url = uri.to_string();
    let host = headers
        .get("host")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("localhost");

    info!("Handling request: {} {}", method, original_url);

    // Get domain configuration
    let domain_config = state.get_domain_config(host).await?;
    debug!("Found domain config for: {}", host);

    // Handle CORS preflight
    if method == Method::OPTIONS {
        return Ok(handle_cors_preflight());
    }

    let path = uri.path();
    let query = uri.query().unwrap_or("");

    // Handle special paths
    if path == "/robots.txt" {
        return Ok(generate_robots_txt(&domain_config));
    }

    if path == "/sitemap.xml" {
        return Ok(generate_sitemap(&domain_config));
    }

    // Parse the original URL and rewrite to target Notion
    let notion_url = format!(
        "https://{}.notion.site{}{}",
        state.config.notion.username,
        path,
        if query.is_empty() {
            String::new()
        } else {
            format!("?{query}")
        }
    );

    debug!("Proxying to: {}", notion_url);

    // Handle different types of requests
    if path.starts_with("/app") && path.ends_with(".js") {
        return handle_js_assets(&state, &notion_url, &domain_config).await;
    }

    if path.starts_with("/api") {
        return handle_api_requests(&state, &notion_url, method, headers, body).await;
    }

    // Check for slug redirects
    let path_slug = path.strip_prefix('/').unwrap_or("");
    if let Some(page_id) = domain_config.slug_to_page.get(path_slug) {
        info!("Redirecting slug '{}' to page '{}'", path_slug, page_id);
        return Ok(redirect_response(&format!(
            "https://{}/{}",
            domain_config.my_domain, page_id
        )));
    }

    // Check if this looks like a Notion page ID not in our mapping
    let page_id_regex = Regex::new(r"^[0-9a-f]{32}$").expect("Valid regex pattern");
    if page_id_regex.is_match(path_slug) && !domain_config.pages.contains(&path_slug.to_string()) {
        info!("Redirecting unknown page ID '{}' to main page", path_slug);
        return Ok(redirect_response(&format!(
            "https://{}",
            domain_config.my_domain
        )));
    }

    // Default: fetch and rewrite HTML content
    handle_html_content(&state, &notion_url, method, headers, body, &domain_config).await
}

fn handle_cors_preflight() -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header("Access-Control-Allow-Origin", "*")
        .header(
            "Access-Control-Allow-Methods",
            "GET, HEAD, POST, PUT, OPTIONS",
        )
        .header("Access-Control-Allow-Headers", "Content-Type")
        .body(Body::empty())
        .expect("Valid response build")
}

fn generate_robots_txt(domain_config: &DomainConfig) -> Response {
    let content = format!("Sitemap: https://{}/sitemap.xml", domain_config.my_domain);
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/plain")
        .body(Body::from(content))
        .expect("Valid response build")
}

fn generate_sitemap(domain_config: &DomainConfig) -> Response {
    let urls: Vec<String> = domain_config
        .slugs
        .iter()
        .map(|slug| {
            format!(
                "<url><loc>https://{}/{}</loc></url>",
                domain_config.my_domain, slug
            )
        })
        .collect();

    let sitemap = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">{}</urlset>"#,
        urls.join("")
    );

    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/xml")
        .body(Body::from(sitemap))
        .expect("Valid response build")
}

fn redirect_response(location: &str) -> Response {
    Response::builder()
        .status(StatusCode::MOVED_PERMANENTLY)
        .header("location", location)
        .body(Body::empty())
        .expect("Valid response build")
}

async fn handle_js_assets(
    state: &AppState,
    notion_url: &str,
    domain_config: &DomainConfig,
) -> Result<Response> {
    let response = state.client.get(notion_url).send().await?;
    let mut body = response.text().await?;

    // Rewrite JavaScript to replace domain references
    body = body
        .replace("www.notion.so", &domain_config.my_domain)
        .replace("notion.so", &domain_config.my_domain)
        .replace(
            &format!("{}.notion.site", state.config.notion.username),
            &domain_config.my_domain,
        );

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/javascript")
        .header("Access-Control-Allow-Origin", "*")
        .body(Body::from(body))
        .expect("Valid response build"))
}

async fn handle_api_requests(
    state: &AppState,
    notion_url: &str,
    method: Method,
    _headers: HeaderMap,
    body: Body,
) -> Result<Response> {
    let mut request_builder = state.client.request(method, notion_url);

    // Set headers
    request_builder = request_builder
        .header("content-type", "application/json;charset=UTF-8")
        .header("user-agent", &state.config.notion.user_agent);

    // Add body for non-GET requests, except for specific API endpoints
    if !notion_url.contains("/api/v3/getPublicPageData") {
        let body_bytes = axum::body::to_bytes(body, usize::MAX)
            .await
            .map_err(|e| ProxyError::Internal(e.to_string()))?;
        request_builder = request_builder.body(body_bytes.to_vec());
    }

    let response = request_builder.send().await?;
    let status = response.status();
    let body_bytes = response.bytes().await?;

    Ok(Response::builder()
        .status(status)
        .header("Access-Control-Allow-Origin", "*")
        .header("content-type", "application/json")
        .body(Body::from(body_bytes))
        .expect("Valid response build"))
}

async fn handle_html_content(
    state: &AppState,
    notion_url: &str,
    method: Method,
    headers: HeaderMap,
    body: Body,
    domain_config: &DomainConfig,
) -> Result<Response> {
    let mut request_builder = state.client.request(method, notion_url);

    // Forward headers (excluding some that might cause issues)
    for (name, value) in headers.iter() {
        let name_str = name.as_str().to_lowercase();
        if !["host", "content-length"].contains(&name_str.as_str()) {
            request_builder = request_builder.header(name, value);
        }
    }

    // Add body if present
    let body_bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(|e| ProxyError::Internal(e.to_string()))?;

    if !body_bytes.is_empty() {
        request_builder = request_builder.body(body_bytes.to_vec());
    }

    let response = request_builder.send().await?;
    let status = response.status();
    let mut response_headers = response.headers().clone();
    let body_text = response.text().await?;

    info!("Fetched content from Notion, rewriting HTML");

    // Remove problematic headers
    response_headers.remove("content-security-policy");
    response_headers.remove("x-content-security-policy");

    // Rewrite HTML content
    let rewriter = HtmlRewriter::new(domain_config.clone());
    let rewritten_html = rewriter.rewrite_html(&body_text)?;

    let mut response_builder = Response::builder().status(status);

    // Add response headers
    for (name, value) in response_headers.iter() {
        response_builder = response_builder.header(name, value);
    }

    Ok(response_builder
        .body(Body::from(rewritten_html))
        .expect("Valid response build"))
}
