/**
 * Notion Site Proxy - Cloudflare Workers Edition
 * 
 * A high-performance Cloudflare Worker that enables hosting Notion pages
 * on custom domains with enhanced features including SEO optimization,
 * dark mode support, custom fonts, and clean URL routing.
 * 
 * @author Akagi201
 * @license Apache-2.0
 * @version 1.0.0
 */

/* ==================== CONFIGURATION SECTION ==================== */

/**
 * Manual configuration object (optional)
 * Uncomment and configure this object to use static configuration
 * instead of KV storage for single-domain setups.
 */
const manualConfig = undefined;
/*
const manualConfig = {
  MY_DOMAIN: 'example.com',
  SLUG_TO_PAGE: {
    "": "homepage-notion-page-id-32-chars",
    "about": "about-page-notion-id-32-chars",
    "contact": "contact-page-notion-id-32-chars",
    "blog": "blog-page-notion-id-32-chars"
  },
  PAGE_TITLE: "Your Site Title",
  PAGE_DESCRIPTION: "Your site description for SEO and social sharing",
  GOOGLE_FONT: "Inter", // Google Font family name
  CUSTOM_SCRIPT: ""     // Custom JavaScript/HTML to inject
};
*/

/**
 * Notion username configuration
 * This should match your Notion workspace username (from yourname.notion.site)
 */
const MY_NOTION_USERNAME = 'faeton';

/**
 * Domain configuration cache
 * Stores resolved configurations to improve performance
 */
const domainConfigCache = new Map();

/* ==================== CORE FUNCTIONS ==================== */

/**
 * Retrieves and processes domain configuration
 * @param {string} hostname - The incoming hostname
 * @returns {Promise<Object|null>} Domain configuration object or null
 */
async function getDomainConfig(hostname) {
  // Remove www prefix for consistency
  const cleanHostname = hostname.replace(/^www\./, '');
  
  // Use manual configuration if defined
  if (typeof manualConfig !== 'undefined' && manualConfig !== null) {
    const config = JSON.parse(JSON.stringify(manualConfig));
    config.MY_DOMAIN = cleanHostname;
    return processConfigurationObject(config);
  }
  
  // Check cache first
  if (domainConfigCache.has(cleanHostname)) {
    return domainConfigCache.get(cleanHostname);
  }
  
  try {
    // Fetch from KV storage (requires DOMAINS_CONFIG KV namespace binding)
    // @ts-ignore: KV binding for DOMAINS_CONFIG
    const domainConfigData = await DOMAINS_CONFIG.get(cleanHostname);
    if (!domainConfigData) {
      console.warn(`No configuration found for domain: ${cleanHostname}`);
      return null;
    }
    
    const config = JSON.parse(domainConfigData);
    config.MY_DOMAIN = cleanHostname;
    const processedConfig = processConfigurationObject(config);
    
    // Cache the processed configuration
    domainConfigCache.set(cleanHostname, processedConfig);
    return processedConfig;
  } catch (error) {
    console.error(`Failed to load configuration for ${cleanHostname}:`, error);
    return null;
  }
}

/**
 * Processes and enriches configuration object with derived fields
 * @param {Object} config - Raw configuration object
 * @returns {Object} Processed configuration with computed fields
 */
function processConfigurationObject(config) {
  config.PAGE_TO_SLUG = {};
  config.slugs = [];
  config.pages = [];
  
  // Build reverse mappings and arrays for quick lookups
  Object.entries(config.SLUG_TO_PAGE || {}).forEach(([slug, pageId]) => {
    if (pageId && typeof pageId === 'string') {
      config.slugs.push(slug);
      config.pages.push(pageId);
      config.PAGE_TO_SLUG[pageId] = slug;
    }
  });
  
  return config;
}

/**
 * Generates XML sitemap based on configured slugs
 * @param {Object} param0 - Configuration object with MY_DOMAIN and slugs
 * @returns {string} XML sitemap content
 */
function generateSitemap({ MY_DOMAIN, slugs = [] }) {
  const urls = slugs
    .filter(slug => slug !== undefined)
    .map(slug => {
      const loc = slug === '' ? `https://${MY_DOMAIN}/` : `https://${MY_DOMAIN}/${slug}`;
      return `<url><loc>${loc}</loc><changefreq>weekly</changefreq><priority>0.8</priority></url>`;
    });
  
  return `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
${urls.join('\n')}
</urlset>`;
}

/**
 * CORS headers for cross-origin requests
 */
const corsHeaders = {
  "Access-Control-Allow-Origin": "*",
  "Access-Control-Allow-Methods": "GET, HEAD, POST, PUT, OPTIONS",
  "Access-Control-Allow-Headers": "Content-Type, Authorization"
};

/**
 * Handles CORS preflight requests
 * @param {Request} request - The incoming request
 * @returns {Response} CORS preflight response
 */
function handleCorsPreflightRequest(request) {
  const origin = request.headers.get("Origin");
  const method = request.headers.get("Access-Control-Request-Method");
  const headers = request.headers.get("Access-Control-Request-Headers");
  
  if (origin && method && headers) {
    return new Response(null, { 
      status: 204,
      headers: corsHeaders 
    });
  }
  
  return new Response(null, { 
    status: 405,
    headers: { 
      Allow: "GET, HEAD, POST, PUT, OPTIONS" 
    } 
  });
}

/* ==================== MAIN REQUEST HANDLER ==================== */

/**
 * Main request handler and entry point
 * Processes incoming requests and routes them appropriately
 * @param {Request} request - The incoming HTTP request
 * @returns {Promise<Response>} HTTP response
 */
async function fetchAndApply(request) {
  try {
    const url = new URL(request.url);
    const hostname = url.hostname.replace(/^www\./, '');
    
    console.log(`Processing request for ${hostname}${url.pathname}`);
    
    // Load domain configuration
    const config = await getDomainConfig(hostname);
    if (!config) {
      return new Response(`Domain "${hostname}" not configured`, { 
        status: 404,
        headers: { 'Content-Type': 'text/plain' }
      });
    }
    
    // Handle CORS preflight
    if (request.method === "OPTIONS") {
      return handleCorsPreflightRequest(request);
    }
    
    // Rewrite URL to target Notion
    const notionUrl = new URL(url.toString());
    notionUrl.hostname = `${MY_NOTION_USERNAME}.notion.site`;
    
    // Handle special endpoints
    if (url.pathname === "/robots.txt") {
      return generateRobotsTxt(config);
    }
    
    if (url.pathname === "/sitemap.xml") {
      return generateSitemapResponse(config);
    }
    
    if (url.pathname === "/health" || url.pathname === "/_health") {
      return new Response("OK", { 
        status: 200,
        headers: { 'Content-Type': 'text/plain' }
      });
    }
    
    // Route different types of requests
    if (url.pathname.startsWith("/app") && url.pathname.endsWith(".js")) {
      return handleJavaScriptAssets(notionUrl, config);
    }
    
    if (url.pathname.startsWith("/api")) {
      return handleApiRequests(notionUrl, request);
    }
    
    // Handle custom slug redirections
    const slugRedirectResponse = handleSlugRedirection(url, config);
    if (slugRedirectResponse) {
      return slugRedirectResponse;
    }
    
    // Handle unknown Notion page IDs
    const unknownPageRedirectResponse = handleUnknownPageRedirection(url, config);
    if (unknownPageRedirectResponse) {
      return unknownPageRedirectResponse;
    }
    
    // Default: fetch and enhance HTML content
    return await handleHtmlContent(notionUrl, request, config);
    
  } catch (error) {
    console.error('Request handling error:', error);
    return new Response(`Internal Server Error: ${error.message}`, { 
      status: 500,
      headers: { 'Content-Type': 'text/plain' }
    });
  }
}

/* ==================== RESPONSE HANDLERS ==================== */

/**
 * Generates robots.txt response
 * @param {Object} config - Domain configuration
 * @returns {Response} robots.txt response
 */
function generateRobotsTxt(config) {
  const content = `User-agent: *
Allow: /

Sitemap: https://${config.MY_DOMAIN}/sitemap.xml`;
  
  return new Response(content, {
    headers: { 'Content-Type': 'text/plain' }
  });
}

/**
 * Generates sitemap.xml response
 * @param {Object} config - Domain configuration
 * @returns {Response} sitemap.xml response
 */
function generateSitemapResponse(config) {
  const sitemap = generateSitemap(config);
  return new Response(sitemap, {
    headers: { 'Content-Type': 'application/xml' }
  });
}

/**
 * Handles JavaScript asset requests with domain rewriting
 * @param {URL} notionUrl - Target Notion URL
 * @param {Object} config - Domain configuration
 * @returns {Promise<Response>} Modified JavaScript response
 */
async function handleJavaScriptAssets(notionUrl, config) {
  try {
    const response = await fetch(notionUrl.toString());
    if (!response.ok) {
      throw new Error(`Failed to fetch JS asset: ${response.status}`);
    }
    
    let body = await response.text();
    
    // Replace domain references in JavaScript
    body = body
      .replace(/www\.notion\.so/g, config.MY_DOMAIN)
      .replace(/notion\.so/g, config.MY_DOMAIN)
      .replace(new RegExp(`${MY_NOTION_USERNAME}\\.notion\\.site`, 'g'), config.MY_DOMAIN);
    
    return new Response(body, {
      status: response.status,
      headers: {
        ...corsHeaders,
        'Content-Type': 'application/javascript',
        'Cache-Control': 'public, max-age=31536000'
      }
    });
  } catch (error) {
    console.error('JS asset handling error:', error);
    return new Response('Failed to load JavaScript asset', { status: 500 });
  }
}

/**
 * Handles Notion API requests with proper forwarding
 * @param {URL} notionUrl - Target Notion URL
 * @param {Request} request - Original request
 * @returns {Promise<Response>} API response
 */
async function handleApiRequests(notionUrl, request) {
  try {
    const requestInit = {
      method: "POST",
      headers: {
        "Content-Type": "application/json;charset=UTF-8",
        "User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36"
      }
    };
    
    // Add body for non-getPublicPageData requests
    if (!notionUrl.pathname.includes('/api/v3/getPublicPageData')) {
      requestInit.body = request.body;
    }
    
    const response = await fetch(notionUrl.toString(), requestInit);
    const responseBody = response.body;
    
    return new Response(responseBody, {
      status: response.status,
      headers: {
        ...corsHeaders,
        'Content-Type': 'application/json'
      }
    });
  } catch (error) {
    console.error('API request handling error:', error);
    return new Response('API request failed', { status: 500 });
  }
}

/**
 * Handles custom slug redirections
 * @param {URL} url - Request URL
 * @param {Object} config - Domain configuration
 * @returns {Response|null} Redirect response or null
 */
function handleSlugRedirection(url, config) {
  const slug = url.pathname.slice(1); // Remove leading slash
  const pageId = config.SLUG_TO_PAGE[slug];
  
  if (pageId) {
    console.log(`Redirecting slug "${slug}" to page "${pageId}"`);
    const redirectUrl = slug === '' 
      ? `https://${config.MY_DOMAIN}/` 
      : `https://${config.MY_DOMAIN}/${pageId}`;
    
    return Response.redirect(redirectUrl, 301);
  }
  
  return null;
}

/**
 * Handles unknown Notion page ID redirections
 * @param {URL} url - Request URL
 * @param {Object} config - Domain configuration
 * @returns {Response|null} Redirect response or null
 */
function handleUnknownPageRedirection(url, config) {
  const pathSegment = url.pathname.slice(1);
  const notionPageIdPattern = /^[0-9a-f]{32}$/i;
  
  if (notionPageIdPattern.test(pathSegment) && !config.pages.includes(pathSegment)) {
    console.log(`Redirecting unknown page ID "${pathSegment}" to homepage`);
    return Response.redirect(`https://${config.MY_DOMAIN}/`, 301);
  }
  
  return null;
}

/**
 * Handles HTML content requests with enhancement
 * @param {URL} notionUrl - Target Notion URL
 * @param {Request} request - Original request
 * @param {Object} config - Domain configuration
 * @returns {Promise<Response>} Enhanced HTML response
 */
async function handleHtmlContent(notionUrl, request, config) {
  try {
    console.log(`Fetching HTML content from: ${notionUrl.toString()}`);
    
    const response = await fetch(notionUrl.toString(), {
      method: request.method,
      headers: request.headers,
      body: request.body
    });
    
    if (!response.ok) {
      throw new Error(`Failed to fetch HTML: ${response.status}`);
    }
    
    // Remove problematic security headers
    const enhancedResponse = new Response(response.body, response);
    enhancedResponse.headers.delete("Content-Security-Policy");
    enhancedResponse.headers.delete("X-Content-Security-Policy");
    
    // Apply HTML enhancements
    return await applyHtmlEnhancements(enhancedResponse, config);
  } catch (error) {
    console.error('HTML content handling error:', error);
    return new Response('Failed to load page content', { status: 500 });
  }
}

/* ==================== HTML ENHANCEMENT CLASSES ==================== */

/**
 * Enhances meta tags for SEO optimization
 */
class MetaTagEnhancer {
  constructor({ PAGE_TITLE, PAGE_DESCRIPTION, MY_DOMAIN }) {
    this.PAGE_TITLE = PAGE_TITLE;
    this.PAGE_DESCRIPTION = PAGE_DESCRIPTION;
    this.MY_DOMAIN = MY_DOMAIN;
  }
  
  element(element) {
    // Update page title
    if (this.PAGE_TITLE) {
      if (element.tagName === "title") {
        element.setInnerContent(this.PAGE_TITLE);
      }
      
      // Update Open Graph and Twitter titles
      const titleProperties = ["og:title", "twitter:title"];
      if (titleProperties.includes(element.getAttribute("property")) ||
          titleProperties.includes(element.getAttribute("name"))) {
        element.setAttribute("content", this.PAGE_TITLE);
      }
    }
    
    // Update page description
    if (this.PAGE_DESCRIPTION) {
      const descriptionProperties = ["description", "og:description", "twitter:description"];
      if (descriptionProperties.includes(element.getAttribute("name")) ||
          descriptionProperties.includes(element.getAttribute("property"))) {
        element.setAttribute("content", this.PAGE_DESCRIPTION);
      }
    }
    
    // Update domain references
    const urlProperties = ["og:url", "twitter:url"];
    if (urlProperties.includes(element.getAttribute("property")) ||
        urlProperties.includes(element.getAttribute("name"))) {
      element.setAttribute("content", `https://${this.MY_DOMAIN}`);
    }
    
    // Remove Apple iTunes app promotion
    if (element.getAttribute("name") === "apple-itunes-app") {
      element.remove();
    }
  }
}

/**
 * Enhances head section with fonts and styles
 */
class HeadSectionEnhancer {
  constructor({ GOOGLE_FONT }) {
    this.GOOGLE_FONT = GOOGLE_FONT;
  }
  
  element(element) {
    let headContent = '';
    
    // Add Google Fonts if specified
    if (this.GOOGLE_FONT) {
      const fontUrl = `https://fonts.googleapis.com/css2?family=${encodeURIComponent(this.GOOGLE_FONT)}:wght@300;400;500;600;700&display=swap`;
      headContent += `
        <link href="${fontUrl}" rel="stylesheet">
        <style>
          * { 
            font-family: "${this.GOOGLE_FONT}", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif !important; 
          }
        </style>`;
    }
    
    // Add custom styles for UI cleanup and dark mode support
    headContent += `
      <style>
        /* Hide Notion's default topbar */
        div.notion-topbar,
        div.notion-topbar-mobile { 
          display: none !important; 
        }
        
        /* Show dark mode toggle when present */
        div.notion-topbar > div > div:nth-child(1n).toggle-mode,
        div.notion-topbar-mobile > div:nth-child(1n).toggle-mode { 
          display: block !important; 
        }
        
        /* Dark mode styles */
        body.dark {
          background-color: #1a1a1a !important;
          color: #e4e4e7 !important;
        }
        
        /* Responsive improvements */
        @media (max-width: 768px) {
          .notion-page-content {
            padding: 0 16px !important;
          }
        }
      </style>`;
    
    if (headContent) {
      element.append(headContent, { html: true });
    }
  }
}

/**
 * Enhances body with JavaScript functionality
 */
class BodySectionEnhancer {
  constructor({ SLUG_TO_PAGE, CUSTOM_SCRIPT, MY_DOMAIN, PAGE_TO_SLUG, pages, slugs }) {
    this.SLUG_TO_PAGE = SLUG_TO_PAGE || {};
    this.CUSTOM_SCRIPT = CUSTOM_SCRIPT || '';
    this.MY_DOMAIN = MY_DOMAIN;
    this.PAGE_TO_SLUG = PAGE_TO_SLUG || {};
    this.pages = pages || [];
    this.slugs = slugs || [];
  }
  
  element(element) {
    const script = `
      <script>
        (function() {
          'use strict';
          
          // Configuration
          window.CONFIG = window.CONFIG || {};
          window.CONFIG.domainBaseUrl = location.origin;
          
          const SLUG_TO_PAGE = ${JSON.stringify(this.SLUG_TO_PAGE)};
          const PAGE_TO_SLUG = ${JSON.stringify(this.PAGE_TO_SLUG)};
          const slugs = ${JSON.stringify(this.slugs)};
          const pages = ${JSON.stringify(this.pages)};
          
          // State management
          let isInitialized = false;
          const darkModeToggle = document.createElement('div');
          
          // Utility functions
          function getCurrentPageId() { 
            return location.pathname.slice(-32); 
          }
          
          function getCurrentSlug() { 
            return location.pathname.slice(1); 
          }
          
          function updateUrlSlug() {
            const pageId = getCurrentPageId();
            const slug = PAGE_TO_SLUG[pageId];
            if (slug !== undefined) {
              const newPath = slug === '' ? '/' : '/' + slug;
              if (location.pathname !== newPath) {
                history.replaceState(history.state, '', newPath);
              }
            }
          }
          
          // Dark mode functions
          function enableDarkMode() {
            darkModeToggle.innerHTML = \`
              <div title="Switch to Light Mode" style="margin: auto 14px 0 0; min-width: 0;">
                <div role="button" tabindex="0" style="user-select: none; transition: background 120ms ease-in; cursor: pointer; border-radius: 44px;">
                  <div style="display: flex; height: 14px; width: 26px; border-radius: 44px; padding: 2px; background: rgb(46, 170, 220); transition: background 200ms ease, box-shadow 200ms ease;">
                    <div style="width: 14px; height: 14px; border-radius: 44px; background: white; transition: transform 200ms ease-out, background 200ms ease-out; transform: translateX(12px);"></div>
                  </div>
                </div>
              </div>\`;
            document.body.classList.add('dark');
            localStorage.setItem('notion-dark-mode', 'true');
            
            // Update Notion's theme if available
            if (window.__console?.environment?.ThemeStore) {
              window.__console.environment.ThemeStore.setState({ mode: 'dark' });
            }
          }
          
          function enableLightMode() {
            darkModeToggle.innerHTML = \`
              <div title="Switch to Dark Mode" style="margin: auto 14px 0 0; min-width: 0;">
                <div role="button" tabindex="0" style="user-select: none; transition: background 120ms ease-in; cursor: pointer; border-radius: 44px;">
                  <div style="display: flex; height: 14px; width: 26px; border-radius: 44px; padding: 2px; background: rgba(135, 131, 120, 0.3); transition: background 200ms ease, box-shadow 200ms ease;">
                    <div style="width: 14px; height: 14px; border-radius: 44px; background: white; transition: transform 200ms ease-out, background 200ms ease-out; transform: translateX(0);"></div>
                  </div>
                </div>
              </div>\`;
            document.body.classList.remove('dark');
            localStorage.setItem('notion-dark-mode', 'false');
            
            // Update Notion's theme if available
            if (window.__console?.environment?.ThemeStore) {
              window.__console.environment.ThemeStore.setState({ mode: 'light' });
            }
          }
          
          function toggleDarkMode() {
            document.body.classList.contains('dark') ? enableLightMode() : enableDarkMode();
          }
          
          function initializeDarkModeToggle(device) {
            const navSelector = device === 'web' ? '.notion-topbar' : '.notion-topbar-mobile';
            const nav = document.querySelector(navSelector);
            
            if (!nav) return false;
            
            const navChild = device === 'web' ? nav.firstChild : nav;
            if (!navChild) return false;
            
            darkModeToggle.className = 'toggle-mode';
            darkModeToggle.addEventListener('click', toggleDarkMode);
            navChild.appendChild(darkModeToggle);
            
            // Initialize based on user preference
            const savedMode = localStorage.getItem('notion-dark-mode');
            const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
            
            if (savedMode === 'true' || (savedMode === null && prefersDark)) {
              enableDarkMode();
            } else {
              enableLightMode();
            }
            
            // Listen for system theme changes
            window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
              if (localStorage.getItem('notion-dark-mode') === null) {
                e.matches ? enableDarkMode() : enableLightMode();
              }
            });
            
            return true;
          }
          
          // Navigation handling
          function initializeNavigation() {
            // Override history methods for proper slug handling
            const originalReplaceState = window.history.replaceState;
            window.history.replaceState = function(state, title, url) {
              if (title !== 'bypass' && slugs.includes(getCurrentSlug())) {
                return; // Prevent replacing slug URLs
              }
              return originalReplaceState.apply(window.history, arguments);
            };
            
            const originalPushState = window.history.pushState;
            window.history.pushState = function(state, title, url) {
              if (url) {
                const dest = new URL(url, location.origin);
                const pageId = dest.pathname.slice(-32);
                if (pages.includes(pageId)) {
                  const slug = PAGE_TO_SLUG[pageId];
                  arguments[2] = slug === '' ? '/' : '/' + slug;
                }
              }
              return originalPushState.apply(window.history, arguments);
            };
            
            // Handle popstate events
            const originalPopstate = window.onpopstate;
            window.onpopstate = function(event) {
              const currentSlug = getCurrentSlug();
              if (slugs.includes(currentSlug)) {
                const pageId = SLUG_TO_PAGE[currentSlug];
                if (pageId) {
                  history.replaceState(history.state, 'bypass', '/' + pageId);
                }
              }
              if (originalPopstate) {
                originalPopstate.apply(this, arguments);
              }
              updateUrlSlug();
            };
          }
          
          // XMLHttpRequest override for API calls
          function initializeApiOverrides() {
            const originalOpen = window.XMLHttpRequest.prototype.open;
            window.XMLHttpRequest.prototype.open = function() {
              if (arguments[1]) {
                arguments[1] = arguments[1].replace('${this.MY_DOMAIN}', '${MY_NOTION_USERNAME}.notion.site');
              }
              return originalOpen.apply(this, arguments);
            };
          }
          
          // Main initialization
          function initialize() {
            if (isInitialized) return;
            
            const webNav = document.querySelector('.notion-topbar');
            const mobileNav = document.querySelector('.notion-topbar-mobile');
            
            if ((webNav && webNav.firstChild?.firstChild) || (mobileNav && mobileNav.firstChild)) {
              isInitialized = true;
              
              updateUrlSlug();
              initializeDarkModeToggle(webNav ? 'web' : 'mobile');
              initializeNavigation();
              initializeApiOverrides();
              
              console.log('Notion proxy enhancements initialized');
            }
          }
          
          // Start initialization
          const observer = new MutationObserver(initialize);
          const notionApp = document.querySelector('#notion-app');
          
          if (notionApp) {
            observer.observe(notionApp, { 
              childList: true, 
              subtree: true 
            });
            
            // Try immediate initialization
            initialize();
          }
          
          // Cleanup on page unload
          window.addEventListener('beforeunload', () => {
            observer.disconnect();
          });
        })();
      </script>
      ${this.CUSTOM_SCRIPT}`;
    
    element.append(script, { html: true });
  }
}

/**
 * Applies HTML enhancements using the defined enhancer classes
 * @param {Response} response - Original HTML response
 * @param {Object} config - Domain configuration
 * @returns {Promise<Response>} Enhanced HTML response
 */
async function applyHtmlEnhancements(response, config) {
  // @ts-ignore: HTMLRewriter is available in Cloudflare Workers runtime
  return new HTMLRewriter()
    .on("title", new MetaTagEnhancer(config))
    .on("meta", new MetaTagEnhancer(config))
    .on("head", new HeadSectionEnhancer(config))
    .on("body", new BodySectionEnhancer({
      SLUG_TO_PAGE: config.SLUG_TO_PAGE,
      CUSTOM_SCRIPT: config.CUSTOM_SCRIPT,
      MY_DOMAIN: config.MY_DOMAIN,
      PAGE_TO_SLUG: config.PAGE_TO_SLUG,
      pages: config.pages,
      slugs: config.slugs
    }))
    .transform(response);
}

/* ==================== EVENT LISTENERS ==================== */

// Main fetch event listener  
// @ts-ignore: addEventListener and event.respondWith are available in Cloudflare Workers runtime
addEventListener("fetch", event => {
  event.respondWith(fetchAndApply(event.request));
});
