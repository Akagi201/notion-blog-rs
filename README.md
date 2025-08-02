# notion-site-proxy

A high-performance Notion site proxy server built with Rust and Axum, allowing you to host Notion pages on your custom domain with enhanced features.

## Features

- **Custom Domain Hosting**: Host your Notion pages on your own domain
- **SEO Optimization**: Customize meta tags, page titles, descriptions, and Open Graph tags
- **Dark Mode Support**: Automatic system preference detection with manual toggle
- **Custom Fonts**: Easy Google Fonts integration
- **Clean UI**: Removes Notion's default navigation elements
- **Sitemap Generation**: Automatic sitemap.xml generation for SEO
- **URL Slugs**: Map human-readable slugs to Notion page IDs
- **Multi-domain Support**: Host multiple domains from a single server instance
- **High Performance**: Built with Rust for optimal speed and resource usage
- **Caching**: Intelligent caching system for improved performance
- **Docker Support**: Easy deployment with Docker

## Architecture

This server acts as a reverse proxy between your custom domain and Notion. When a visitor accesses your domain:

1. The server retrieves domain configuration
2. Fetches the corresponding Notion page content
3. Applies custom modifications (SEO meta tags, dark mode, custom fonts)
4. Handles URL routing and slug redirection
5. Returns the enhanced page to the visitor

## Quick Start

### Prerequisites

- Rust 1.70+ (for building from source)
- Or Docker (for containerized deployment)

### Installation

#### From Source

```bash
git clone https://github.com/Akagi201/notion-site-proxy.git
cd notion-site-proxy
cargo build --release
```

#### Using Docker

```bash
docker pull ghcr.io/akagi201/notion-site-proxy:latest
```

### Configuration

1. Copy the example configuration:
```bash
cp config.toml.example config.toml
```

2. Edit `config.toml` with your domain and Notion settings:

```toml
[server]
host = "0.0.0.0"
port = 3000
log_level = "info"

[notion]
username = "your-notion-username"

[domains."yourdomain.com"]
my_domain = "yourdomain.com"
page_title = "Your Site Title"
page_description = "Your site description for SEO"
google_font = "Inter"

[domains."yourdomain.com".slug_to_page]
"" = "your-homepage-notion-page-id"
"about" = "your-about-page-notion-id"
"contact" = "your-contact-page-notion-id"
```

### Running

#### From Source
```bash
./target/release/notion-site-proxy --config config.toml
```

#### Using Docker
```bash
docker run -p 3000:3000 -v $(pwd)/config.toml:/app/config.toml ghcr.io/akagi201/notion-site-proxy:latest
```

## Configuration Reference

### Server Configuration

```toml
[server]
host = "0.0.0.0"           # Server bind address
port = 3000                # Server port
log_level = "info"         # Log level: trace, debug, info, warn, error
```

### Notion Configuration

```toml
[notion]
username = "your-username"  # Your Notion username (from your notion.site URL)
user_agent = "..."         # User agent for Notion requests
```

### Cache Configuration

```toml
[cache]
max_capacity = 1000        # Maximum number of cached items
time_to_live_secs = 3600   # Cache TTL in seconds
```

### Domain Configuration

```toml
[domains."example.com"]
my_domain = "example.com"
page_title = "Optional custom title"
page_description = "Optional meta description"
google_font = "Inter"      # Optional Google Font name
custom_script = ""         # Optional custom JavaScript

[domains."example.com".slug_to_page]
"" = "notion-page-id-for-homepage"
"about" = "notion-page-id-for-about"
"blog" = "notion-page-id-for-blog"
```

## Command Line Options

```bash
notion-site-proxy [OPTIONS]

OPTIONS:
    -c, --config <FILE>      Configuration file path [default: config.toml]
    -h, --host <HOST>        Override server host
    -p, --port <PORT>        Override server port
        --log-level <LEVEL>  Override log level
        --help               Print help information
        --version            Print version information
```

## URL Mapping

The server supports flexible URL mapping:

- `yourdomain.com/` → Notion page mapped to empty slug `""`
- `yourdomain.com/about` → Notion page mapped to `"about"` slug
- `yourdomain.com/blog` → Notion page mapped to `"blog"` slug
- Direct Notion page IDs are automatically redirected to their mapped slugs
- Unknown Notion page IDs redirect to the homepage

## Special Endpoints

- `/robots.txt` - Automatically generated robots.txt with sitemap reference
- `/sitemap.xml` - Automatically generated XML sitemap
- `/health` - Health check endpoint

## Deployment

### Docker Compose

```yaml
version: '3.8'
services:
  notion-proxy:
    image: ghcr.io/akagi201/notion-site-proxy:latest
    ports:
      - "3000:3000"
    volumes:
      - ./config.toml:/app/config.toml:ro
    restart: unless-stopped
```

### Reverse Proxy Setup

#### Nginx

```nginx
server {
    listen 80;
    server_name yourdomain.com;
    
    location / {
        proxy_pass http://localhost:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

#### Cloudflare

Set up Cloudflare to point to your server IP, and the proxy will handle the rest.

## How It Works

### HTML Rewriting

The server performs several HTML modifications:

1. **Meta Tag Injection**: Updates title, description, and Open Graph tags
2. **Font Integration**: Adds Google Fonts CSS links
3. **Dark Mode**: Injects dark mode toggle functionality
4. **URL Rewriting**: Replaces Notion URLs with your custom domain
5. **Navigation Removal**: Hides Notion's default topbar

### Caching Strategy

- Domain configurations are cached in memory
- HTTP responses can be cached (configurable)
- Cache invalidation based on TTL

### Error Handling

- Graceful fallbacks for missing pages
- Comprehensive logging for debugging
- Health checks for monitoring

## Development

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Running with hot reload

```bash
cargo watch -x run
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by the original [Cloudflare Worker implementation](https://github.com/faeton/notionworker)
- Built with the excellent [Axum](https://github.com/tokio-rs/axum) web framework
- HTML processing powered by [scraper](https://github.com/causal-agent/scraper)

## Refs

* <https://github.com/seadfeng/cloudflare-proxy-sites>
* <https://github.com/faeton/notionworker>
