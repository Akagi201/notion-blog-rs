use crate::{
    config::DomainConfig,
    error::{ProxyError, Result},
};

pub struct HtmlRewriter {
    config: DomainConfig,
}

impl HtmlRewriter {
    pub fn new(config: DomainConfig) -> Self {
        Self { config }
    }

    pub fn rewrite_html(&self, html: &str) -> Result<String> {
        // Create a mutable HTML string to work with
        let mut html_content = html.to_string();

        // Rewrite meta tags
        html_content = self.rewrite_meta_tags(&html_content)?;

        // Inject head content
        html_content = self.inject_head_content(&html_content)?;

        // Inject body scripts
        html_content = self.inject_body_scripts(&html_content)?;

        Ok(html_content)
    }

    fn rewrite_meta_tags(&self, html: &str) -> Result<String> {
        let mut html_content = html.to_string();

        // Replace page title
        if let Some(page_title) = &self.config.page_title {
            // Replace title tag
            html_content = html_content.replace(
                r#"<title>"#,
                &format!(r#"<title>{page_title}</title><title style="display:none">"#,),
            );

            // Replace og:title
            let og_title_pattern =
                regex::Regex::new(r#"<meta\s+property="og:title"\s+content="[^"]*""#)
                    .map_err(|e| ProxyError::HtmlRewrite(e.to_string()))?;
            html_content = og_title_pattern
                .replace_all(
                    &html_content,
                    &format!(r#"<meta property="og:title" content="{page_title}""#),
                )
                .to_string();

            // Replace twitter:title
            let twitter_title_pattern =
                regex::Regex::new(r#"<meta\s+name="twitter:title"\s+content="[^"]*""#)
                    .map_err(|e| ProxyError::HtmlRewrite(e.to_string()))?;
            html_content = twitter_title_pattern
                .replace_all(
                    &html_content,
                    &format!(r#"<meta name="twitter:title" content="{page_title}""#),
                )
                .to_string();
        }

        // Replace page description
        if let Some(page_description) = &self.config.page_description {
            let desc_patterns = [
                (
                    r#"<meta\s+name="description"\s+content="[^"]*""#,
                    r#"<meta name="description" content="{}""#,
                ),
                (
                    r#"<meta\s+property="og:description"\s+content="[^"]*""#,
                    r#"<meta property="og:description" content="{}""#,
                ),
                (
                    r#"<meta\s+name="twitter:description"\s+content="[^"]*""#,
                    r#"<meta name="twitter:description" content="{}""#,
                ),
            ];

            for (pattern, replacement) in desc_patterns {
                let regex = regex::Regex::new(pattern)
                    .map_err(|e| ProxyError::HtmlRewrite(e.to_string()))?;
                html_content = regex
                    .replace_all(&html_content, &replacement.replace("{}", page_description))
                    .to_string();
            }
        }

        // Replace domain references
        let domain_patterns = [
            (
                r#"<meta\s+property="og:url"\s+content="[^"]*""#,
                format!(
                    r#"<meta property="og:url" content="{}""#,
                    self.config.my_domain
                ),
            ),
            (
                r#"<meta\s+name="twitter:url"\s+content="[^"]*""#,
                format!(
                    r#"<meta name="twitter:url" content="{}""#,
                    self.config.my_domain
                ),
            ),
        ];

        for (pattern, replacement) in domain_patterns {
            let regex =
                regex::Regex::new(pattern).map_err(|e| ProxyError::HtmlRewrite(e.to_string()))?;
            html_content = regex
                .replace_all(&html_content, replacement.as_str())
                .to_string();
        }

        // Remove apple-itunes-app meta tag
        let itunes_pattern = regex::Regex::new(r#"<meta\s+name="apple-itunes-app"[^>]*>"#)
            .map_err(|e| ProxyError::HtmlRewrite(e.to_string()))?;
        html_content = itunes_pattern.replace_all(&html_content, "").to_string();

        Ok(html_content)
    }

    fn inject_head_content(&self, html: &str) -> Result<String> {
        let mut head_content = String::new();

        // Add Google Font if specified
        if let Some(google_font) = &self.config.google_font {
            let font_url = format!(
                "https://fonts.googleapis.com/css?family={}:Regular,Bold,Italic&display=swap",
                google_font.replace(' ', "+")
            );
            head_content.push_str(&format!(
                r#"<link href="{font_url}" rel="stylesheet">
                <style>* {{ font-family: "{google_font}" !important; }}</style>"#,
            ));
        }

        // Add custom styles to hide Notion topbar
        head_content.push_str(r#"
            <style>
                div.notion-topbar,
                div.notion-topbar-mobile { display: none !important; }
                div.notion-topbar > div > div:nth-child(1n).toggle-mode,
                div.notion-topbar-mobile > div:nth-child(1n).toggle-mode { display: block !important; }
            </style>
        "#);

        // Inject before closing head tag
        let html_content = html.replace("</head>", &format!("{head_content}</head>"));

        Ok(html_content)
    }

    fn inject_body_scripts(&self, html: &str) -> Result<String> {
        let slug_to_page_json =
            serde_json::to_string(&self.config.slug_to_page).map_err(ProxyError::JsonParse)?;
        let page_to_slug_json =
            serde_json::to_string(&self.config.page_to_slug).map_err(ProxyError::JsonParse)?;
        let slugs_json =
            serde_json::to_string(&self.config.slugs).map_err(ProxyError::JsonParse)?;
        let pages_json =
            serde_json::to_string(&self.config.pages).map_err(ProxyError::JsonParse)?;

        let custom_script = self.config.custom_script.as_deref().unwrap_or("");

        let script_content = format!(
            r#"
            <script>
                window.CONFIG = window.CONFIG || {{}};
                window.CONFIG.domainBaseUrl = location.origin;
                const SLUG_TO_PAGE = {};
                const PAGE_TO_SLUG = {};
                const slugs = {};
                const pages = {};
                const el = document.createElement('div');
                let redirected = false;
                
                function getPage() {{ return location.pathname.slice(-32); }}
                function getSlug() {{ return location.pathname.slice(1); }}
                
                function updateSlug() {{
                    const slug = PAGE_TO_SLUG[getPage()];
                    if (slug != null) history.replaceState(history.state, '', '/' + slug);
                }}
                
                function onDark() {{
                    el.innerHTML = '<div title="Change to Light Mode" style="margin: auto 14px 0 0; min-width: 0;"><div role="button" tabindex="0" style="user-select: none; transition: background 120ms ease-in; cursor: pointer; border-radius: 44px;"><div style="display: flex; height: 14px; width: 26px; border-radius: 44px; padding: 2px; background: rgb(46, 170, 220); transition: background 200ms ease, box-shadow 200ms ease;"><div style="width: 14px; height: 14px; border-radius: 44px; background: white; transition: transform 200ms ease-out, background 200ms ease-out; transform: translateX(12px);"></div></div></div></div>';
                    document.body.classList.add('dark');
                    if (window.__console && window.__console.environment && window.__console.environment.ThemeStore) {{
                        window.__console.environment.ThemeStore.setState({{ mode: 'dark' }});
                    }}
                }}
                
                function onLight() {{
                    el.innerHTML = '<div title="Change to Dark Mode" style="margin: auto 14px 0 0; min-width: 0;"><div role="button" tabindex="0" style="user-select: none; transition: background 120ms ease-in; cursor: pointer; border-radius: 44px;"><div style="display: flex; height: 14px; width: 26px; border-radius: 44px; padding: 2px; background: rgba(135, 131, 120, 0.3); transition: background 200ms ease, box-shadow 200ms ease;"><div style="width: 14px; height: 14px; border-radius: 44px; background: white; transition: transform 200ms ease-out, background 200ms ease-out; transform: translateX(0);"></div></div></div></div>';
                    document.body.classList.remove('dark');
                    if (window.__console && window.__console.environment && window.__console.environment.ThemeStore) {{
                        window.__console.environment.ThemeStore.setState({{ mode: 'light' }});
                    }}
                }}
                
                function toggle() {{ 
                    document.body.classList.contains('dark') ? onLight() : onDark(); 
                }}
                
                function addDarkModeButton(device) {{
                    const nav = device === 'web'
                        ? document.querySelector('.notion-topbar').firstChild
                        : document.querySelector('.notion-topbar-mobile');
                    el.className = 'toggle-mode';
                    el.addEventListener('click', toggle);
                    nav.appendChild(el);
                    if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {{
                        onDark();
                    }} else {{
                        onLight();
                    }}
                    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', toggle);
                }}
                
                const observer = new MutationObserver(() => {{
                    if (redirected) return;
                    const nav = document.querySelector('.notion-topbar');
                    const mobileNav = document.querySelector('.notion-topbar-mobile');
                    if ((nav && nav.firstChild && nav.firstChild.firstChild) || (mobileNav && mobileNav.firstChild)) {{
                        redirected = true;
                        updateSlug();
                        addDarkModeButton(nav ? 'web' : 'mobile');
                        const onpopstate = window.onpopstate;
                        window.onpopstate = function() {{
                            if (slugs.includes(getSlug())) {{
                                const page = SLUG_TO_PAGE[getSlug()];
                                if (page) history.replaceState(history.state, 'bypass', '/' + page);
                            }}
                            onpopstate.apply(this, arguments);
                            updateSlug();
                        }};
                    }}
                }});
                observer.observe(document.querySelector('#notion-app'), {{ childList: true, subtree: true }});
                
                const originalReplaceState = window.history.replaceState;
                window.history.replaceState = function(state) {{
                    if (arguments[1] !== 'bypass' && slugs.includes(getSlug())) return;
                    return originalReplaceState.apply(window.history, arguments);
                }};
                
                const originalPushState = window.history.pushState;
                window.history.pushState = function(state) {{
                    const dest = new URL(location.protocol + '//' + location.host + arguments[2]);
                    const id = dest.pathname.slice(-32);
                    if (pages.includes(id)) arguments[2] = '/' + PAGE_TO_SLUG[id];
                    return originalPushState.apply(window.history, arguments);
                }};
                
                const open = window.XMLHttpRequest.prototype.open;
                window.XMLHttpRequest.prototype.open = function() {{
                    arguments[1] = arguments[1].replace('{}', '{}.notion.site');
                    return open.apply(this, arguments);
                }};
            </script>{}
        "#,
            slug_to_page_json,
            page_to_slug_json,
            slugs_json,
            pages_json,
            self.config.my_domain,
            self.config.my_domain.replace(".notion.site", ""),
            custom_script
        );

        // Inject before closing body tag
        let html_content = html.replace("</body>", &format!("{script_content}</body>"));

        Ok(html_content)
    }
}
