use std::path::PathBuf;

struct Config {
    config_path: PathBuf,
    sort: crate::SortType,
    queries: i32,
    timeout: f64,
    dns_servers: Vec<crate::DNS>,
    test_domains: Vec<String>,
}

static mut CONFIG: Option<Config> = None;

pub fn init(config_path: PathBuf, sort: crate::SortType, queries: i32, timeout: f64) {
    unsafe {
        if CONFIG.is_none() {
            CONFIG = Some(Config {
                config_path,
                sort,
                queries,
                timeout,
                dns_servers: Vec::new(),
                test_domains: Vec::new(),
            });
        }
    }
}

pub fn get_path() -> Option<PathBuf> {
    unsafe { CONFIG.as_ref().map(|cfg| cfg.config_path.clone()) }
}

pub fn update_path(path: PathBuf) {
    unsafe {
        if let Some(cfg) = &mut CONFIG {
            cfg.config_path = path;
        }
    }
}

pub fn get_sort_type() -> Option<crate::SortType> {
    unsafe { CONFIG.as_ref().map(|cfg| cfg.sort) }
}

pub fn get_queries() -> Option<i32> {
    unsafe { CONFIG.as_ref().map(|cfg| cfg.queries) }
}

pub fn get_timeout() -> Option<f64> {
    unsafe { CONFIG.as_ref().map(|cfg| cfg.timeout) }
}

pub fn get_dns_servers() -> Option<Vec<crate::DNS>> {
    unsafe { CONFIG.as_ref().map(|cfg| cfg.dns_servers.clone()) }
}

pub fn add_dns_server(dns: crate::DNS) {
    unsafe {
        if let Some(cfg) = &mut CONFIG {
            cfg.dns_servers.push(dns);
        }
    }
}

pub fn add_test_domain(domain: String) {
    unsafe {
        if let Some(cfg) = &mut CONFIG {
            cfg.test_domains.push(domain)
        }
    }
}

pub fn get_test_domains() -> Option<Vec<String>> {
    unsafe { CONFIG.as_ref().map(|cfg| cfg.test_domains.clone()) }
}
