use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub notion: NotionConfig,
    pub domains: HashMap<String, DomainConfig>,
    pub cache: CacheConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub log_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotionConfig {
    pub username: String,
    pub user_agent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainConfig {
    pub my_domain: String,
    pub slug_to_page: HashMap<String, String>,
    pub page_title: Option<String>,
    pub page_description: Option<String>,
    pub google_font: Option<String>,
    pub custom_script: Option<String>,
    // Computed fields
    #[serde(skip)]
    pub page_to_slug: HashMap<String, String>,
    #[serde(skip)]
    pub slugs: Vec<String>,
    #[serde(skip)]
    pub pages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub max_capacity: u64,
    pub time_to_live_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 3000,
                log_level: "info".to_string(),
            },
            notion: NotionConfig {
                username: "faeton".to_string(),
                user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_12_6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/80.0.3987.163 Safari/537.36".to_string(),
            },
            domains: HashMap::new(),
            cache: CacheConfig {
                max_capacity: 1000,
                time_to_live_secs: 3600,
            },
        }
    }
}

impl DomainConfig {
    #[allow(dead_code)]
    pub fn new(my_domain: String, slug_to_page: HashMap<String, String>) -> Self {
        let mut config = Self {
            my_domain,
            slug_to_page,
            page_title: None,
            page_description: None,
            google_font: None,
            custom_script: None,
            page_to_slug: HashMap::new(),
            slugs: Vec::new(),
            pages: Vec::new(),
        };
        config.compute_derived_fields();
        config
    }

    pub fn compute_derived_fields(&mut self) {
        self.page_to_slug.clear();
        self.slugs.clear();
        self.pages.clear();

        for (slug, page) in &self.slug_to_page {
            self.slugs.push(slug.clone());
            self.pages.push(page.clone());
            self.page_to_slug.insert(page.clone(), slug.clone());
        }
    }
}
