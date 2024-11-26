use std::path::PathBuf;

struct Config {
    config_path: PathBuf,
    sort: crate::SortType,
    queries: u64,
    timeout: f64,
    dns_servers: Vec<crate::DNS>,
    test_domains: Vec<String>,
}

static mut CONFIG: Option<Config> = None;

pub fn init(config_path: PathBuf, sort: crate::SortType, queries: u64, timeout: f64) {
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
    load_data();
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

pub fn get_queries() -> Option<u64> {
    unsafe { CONFIG.as_ref().map(|cfg| cfg.queries) }
}

pub fn get_timeout() -> Option<f64> {
    unsafe { CONFIG.as_ref().map(|cfg| cfg.timeout) }
}

pub fn get_dns_servers() -> Option<Vec<crate::DNS>> {
    unsafe { CONFIG.as_ref().map(|cfg| cfg.dns_servers.clone()) }
}

pub fn add_dns_server(mut dns: crate::DNS) {
    for ip in dns.ips.iter_mut() {
        ip.push_str(":53");
    }

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

fn load_default_data() {
    add_dns_server(crate::DNS {
        name: "google".to_string(),
        ips: vec!["8.8.8.8".to_string(), "8.8.4.4".into()],
    });
    add_dns_server(crate::DNS {
        name: "cloudfare".to_string(),
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
