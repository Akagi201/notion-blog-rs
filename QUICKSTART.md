# Notion Site Proxy - Quick Start Guide

## What is this?

`notion-site-proxy` is a high-performance Rust server that allows you to host your Notion pages on your custom domain. It acts as a proxy between your domain and Notion, while adding custom features like:

- SEO optimization (custom meta tags, titles, descriptions)
- Dark mode toggle
- Custom Google Fonts
- Clean UI (removes Notion branding)
- Custom URL slugs
- Automatic sitemap generation

## Quick Example

Let's say you have a Notion workspace and want to host it on `myblog.com`:

1. **Find your Notion username** - This is from your Notion public URL: `https://yourname.notion.site`

2. **Get your Notion page IDs** - These are the long strings in your Notion URLs:
   ```
   https://yourname.notion.site/About-Me-a1b2c3d4e5f6789012345678901234567890
                                        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
                                        This is your page ID
   ```

3. **Configure the server**:
   ```toml
   [notion]
   username = "yourname"  # from yourname.notion.site
   
   [domains."myblog.com"]
   my_domain = "myblog.com"
   page_title = "My Amazing Blog"
   page_description = "Welcome to my blog powered by Notion"
   google_font = "Inter"
   
   [domains."myblog.com".slug_to_page]
   "" = "a1b2c3d4e5f6789012345678901234567890"        # Homepage
   "about" = "b2c3d4e5f6789012345678901234567890a"    # About page
   "blog" = "c3d4e5f6789012345678901234567890ab"      # Blog page
   ```

4. **Run the server**:
   ```bash
   ./notion-site-proxy --config config.toml
   ```

5. **Point your domain** to the server (port 3000 by default)

Now visitors to `myblog.com` will see your Notion content with:
- `myblog.com/` → Your homepage
- `myblog.com/about` → Your about page  
- `myblog.com/blog` → Your blog page

## Advanced Features

### Multiple Domains
Host multiple sites from one server:

```toml
[domains."site1.com"]
my_domain = "site1.com"
# ... configuration for site1

[domains."site2.com"] 
my_domain = "site2.com"
# ... configuration for site2
```

### Custom JavaScript
Add analytics, chat widgets, or custom functionality:

```toml
[domains."myblog.com"]
custom_script = '''
<script>
  // Google Analytics
  gtag('config', 'GA_TRACKING_ID');
</script>
'''
```

### Docker Deployment
```bash
docker run -p 3000:3000 -v $(pwd)/config.toml:/app/config.toml ghcr.io/akagi201/notion-blog-rs:latest
```

## Architecture

```
Visitor Request → Your Domain → notion-site-proxy → Notion → Enhanced Response
```

The proxy:
1. Receives requests to your domain
2. Fetches content from Notion
3. Applies your customizations (SEO, styling, scripts)
4. Returns the enhanced page to the visitor

## Why Use This?

- **Professional URLs**: `yourdomain.com/about` instead of `yourname.notion.site/a1b2c3...`
- **SEO Control**: Custom titles, descriptions, and meta tags
- **Branding**: Remove Notion UI, add your own styling
- **Performance**: Fast Rust server with intelligent caching
- **Features**: Dark mode, custom fonts, analytics integration

## Support

- 📖 [Full Documentation](README.md)
- 🐛 [Report Issues](https://github.com/Akagi201/notion-blog-rs/issues)
- 💬 [Discussions](https://github.com/Akagi201/notion-blog-rs/discussions)
