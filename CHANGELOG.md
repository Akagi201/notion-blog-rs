# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-08-03

### Added
- Initial release of notion-site-proxy
- Core proxy functionality for Notion sites
- HTML rewriting and injection capabilities
- SEO optimization with custom meta tags
- Dark mode toggle support
- Google Fonts integration
- URL slug mapping to Notion page IDs
- Multi-domain support
- Automatic sitemap.xml generation
- Automatic robots.txt generation
- Configuration file support (TOML format)
- Command-line interface with clap
- Structured logging with tracing
- Docker support with multi-stage builds
- GitHub Actions CI/CD pipeline
- Comprehensive documentation and examples
- MIT license

### Features
- **Proxy Server**: High-performance Rust server built with Axum
- **HTML Rewriting**: Modify Notion pages with custom content
- **SEO**: Custom page titles, descriptions, and Open Graph tags
- **UI Enhancements**: Dark mode toggle and custom fonts
- **URL Management**: Clean, SEO-friendly URLs with custom slugs
- **Multi-tenant**: Support multiple domains from single server
- **Caching**: Intelligent caching system for improved performance
- **Monitoring**: Health check endpoint and structured logging
- **DevOps**: Docker containerization and CI/CD automation

[Unreleased]: https://github.com/Akagi201/notion-blog-rs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Akagi201/notion-blog-rs/releases/tag/v0.1.0
