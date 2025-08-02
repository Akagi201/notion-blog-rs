use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "notion-site-proxy")]
#[command(about = "A Notion site proxy server built with Rust and Axum")]
#[command(version)]
pub struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "config.toml")]
    pub config: String,

    /// Server host to bind to
    #[arg(long)]
    pub host: Option<String>,

    /// Server port to bind to
    #[arg(short, long)]
    pub port: Option<u16>,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    pub log_level: String,
}
