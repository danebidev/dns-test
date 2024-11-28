use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Once, RwLock};

#[derive(Clone, Debug)]
pub struct Config {
    config_path: PathBuf,
    sort: crate::SortType,
    queries: u64,
    dns_servers: Vec<crate::DNS>,
    test_domains: Vec<String>,
}

static INIT: Once = Once::new();
static INITIALIZED: AtomicBool = AtomicBool::new(false);

// Use Arc<RwLock<>> for thread-safe, clone-able configuration
lazy_static::lazy_static! {
    static ref GLOBAL_CONFIG: Arc<RwLock<Option<Config>>> = Arc::new(RwLock::new(None));
}

pub fn init(config_path: PathBuf, sort: crate::SortType, queries: u64) {
    INIT.call_once(|| {
        let mut config = GLOBAL_CONFIG.write().unwrap();
        *config = Some(Config {
            config_path,
            sort,
            queries,
            dns_servers: Vec::new(),
            test_domains: Vec::new(),
        });
        INITIALIZED.store(true, Ordering::SeqCst);
    });

    load_data();
}

fn ensure_initialized() {
    if !INITIALIZED.load(Ordering::SeqCst) {
        panic!("Configuration not initialized. Call init() first.");
    }
}

pub fn get_path() -> Option<PathBuf> {
    ensure_initialized();
    GLOBAL_CONFIG
        .read()
        .unwrap()
        .as_ref()
        .map(|cfg| cfg.config_path.clone())
}

pub fn update_path(path: PathBuf) {
    ensure_initialized();
    if let Ok(mut config) = GLOBAL_CONFIG.write() {
        if let Some(cfg) = config.as_mut() {
            cfg.config_path = path;
        }
    }
}

pub fn get_sort_type() -> Option<crate::SortType> {
    ensure_initialized();
    GLOBAL_CONFIG.read().unwrap().as_ref().map(|cfg| cfg.sort)
}

pub fn get_queries() -> Option<u64> {
    ensure_initialized();
    GLOBAL_CONFIG
        .read()
        .unwrap()
        .as_ref()
        .map(|cfg| cfg.queries)
}

pub fn get_dns_servers() -> Option<Vec<crate::DNS>> {
    ensure_initialized();
    GLOBAL_CONFIG
        .read()
        .unwrap()
        .as_ref()
        .map(|cfg| cfg.dns_servers.clone())
}

pub fn add_dns_server(mut dns: crate::DNS) {
    ensure_initialized();
    for ip in dns.ips.iter_mut() {
        ip.push_str(":53");
    }

    if let Ok(mut config) = GLOBAL_CONFIG.write() {
        if let Some(cfg) = config.as_mut() {
            cfg.dns_servers.push(dns);
        }
    }
}

pub fn add_test_domain(mut domain: String) {
    ensure_initialized();

    if domain.ends_with(".") {
        domain.push_str(".");
    }
    if let Ok(mut config) = GLOBAL_CONFIG.write() {
        if let Some(cfg) = config.as_mut() {
            cfg.test_domains.push(domain)
        }
    }
}

pub fn get_test_domains() -> Option<Vec<String>> {
    ensure_initialized();
    GLOBAL_CONFIG
        .read()
        .unwrap()
        .as_ref()
        .map(|cfg| cfg.test_domains.clone())
}

fn load_default_data() {
    add_dns_server(crate::DNS {
        name: "Google".to_string(),
        ips: vec!["8.8.8.8".to_string(), "8.8.4.4".into()],
    });
    add_dns_server(crate::DNS {
        name: "Cloudflare".to_string(),
        ips: vec!["1.1.1.1".to_string(), "1.0.0.1".into()],
    });

    add_test_domain("google.com".to_string());
    add_test_domain("facebook.com".to_string());
    add_test_domain("amazon.com".to_string());
    add_test_domain("example.com".to_string());
}

fn load_data() {
    load_default_data();
}
