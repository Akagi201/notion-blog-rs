use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("HTTP request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    #[allow(dead_code)]
    Config(String),

    #[error("Domain not found: {0}")]
    DomainNotFound(String),

    #[error("HTML rewriting error: {0}")]
    HtmlRewrite(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Internal server error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, ProxyError>;
