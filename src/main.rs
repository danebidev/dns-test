use std::{
    net::SocketAddr,
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::Instant,
};
use trust_dns_resolver::{
    config::{NameServerConfig, Protocol, ResolverConfig, ResolverOpts},
    Resolver,
};

mod config;

#[derive(Debug)]
struct DNS {
    name: String,
    ips: Vec<String>,
}

impl Clone for DNS {
    fn clone(&self) -> Self {
        DNS {
            name: self.name.clone(),
            ips: self.ips.clone(),
        }
    }
}

#[derive(Debug)]
struct BenchmarkResult {
    min: u128,
    max: u128,
    avg: f64,
}

#[derive(Copy, Clone)]
enum SortType {
    Average,
    Minimum,
    Maximum,
    Lost,
}

fn get_next_arg<'a>(args: &'a Vec<String>, i: &mut usize) -> Option<&'a str> {
    *i += 1;
    if *i >= args.len() || args[*i].starts_with("--") {
        println!("Missing required parameter for option '{}'", args[*i - 1]);
        Option::None
    } else {
        Option::Some(&args[*i])
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let mut config_path: PathBuf;
    match std::env::var("XDG_CONFIG_HOME") {
        Ok(s) => config_path = PathBuf::from(s),
        _ => config_path = PathBuf::from(shellexpand::tilde("~/.config").to_string()),
    };

    config_path.push("dnstest-cli/config.toml");

    let mut sort: SortType = SortType::Average;
    let mut queries: u64 = 100;

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];

        if !arg.starts_with("--") {
            println!("Unrecognized option '{arg}'");
            return;
        }
        let arg_name = &arg[2..];

        match arg_name {
            "config" => {
                let next_arg = get_next_arg(&args, &mut i);

                let path = next_arg.unwrap_or_else(|| std::process::exit(1));
                config_path = PathBuf::from(shellexpand::tilde(path).to_string());
            }
            "sort" => {
                let next_arg = get_next_arg(&args, &mut i);

                let sort_type = next_arg.unwrap_or_else(|| std::process::exit(1));
                match sort_type {
                    "avg" => sort = SortType::Average,
                    "min" => sort = SortType::Minimum,
                    "max" => sort = SortType::Maximum,
                    "lost" => sort = SortType::Lost,
                    _ => {
                        println!("Unrecognized parameter to --sort option '{}'", sort_type);
                        std::process::exit(1);
                    }
                }
            }
            "queries" => {
                let next_arg = get_next_arg(&args, &mut i);
                let next_arg = next_arg.unwrap_or_else(|| std::process::exit(1));

                match next_arg.parse::<u64>() {
                    Ok(parsed) => queries = parsed,
                    Err(_) => {
                        println!("Error parsing parameter to --query option '{}'", next_arg)
                    }
                }
            }
            _ => {
                println!("Unrecognized option '{arg}'");
                return;
            }
        }

        i += 1;
    }

    config::init(config_path, sort, queries);
    run_benchmark();
}

fn dns_lookup(server: &DNS, host: &str) -> u128 {
    let mut config = ResolverConfig::new();
    config.add_name_server(NameServerConfig {
        socket_addr: server.ips[0]
            .parse::<SocketAddr>()
            .expect("Unable to parse socket address"),
        protocol: Protocol::Udp,
        tls_dns_name: None,
        trust_negative_responses: false,
        bind_addr: None,
    });

    let resolver = Resolver::new(config, ResolverOpts::default()).unwrap();

    let start_time = Instant::now();

    match resolver.lookup_ip(host) {
        Ok(_) => start_time.elapsed().as_micros(),
        Err(_) => u128::MAX,
    }
}

fn print_progress_bar(done: usize, total: usize) {
    let to_print = 20 * done / total;
    let mut out = "".to_string();
    for i in 0..20 {
        if i < to_print {
            out.push_str("=");
        } else {
            out.push_str(" ");
        }
    }
    print!("\r[{}] {}% Complete", out, 100 * done / total);
}

fn run_benchmark() {
    let total_lookups = config::get_queries().unwrap();
    let max_concurrent_threads = std::cmp::min(10, total_lookups); // TODO put this in config and get it from user
    let servers = &config::get_dns_servers().unwrap();
    let hosts = &config::get_test_domains().unwrap();
    let mut end_result: Vec<BenchmarkResult> = Vec::new();

    println!("Testing default DNS servers...");

    for (i, server) in servers.into_iter().enumerate() {
        print_progress_bar(i, servers.len());
        let results = Arc::new(Mutex::new(Vec::new()));
        for host in hosts {
            let mut handles = vec![];
            let completed_lookups = Arc::new(Mutex::new(AtomicU64::new(0)));

            for _ in 0..max_concurrent_threads {
                let results = Arc::clone(&results);
                let completed_lookups = Arc::clone(&completed_lookups);
                let host = host.clone();
                let server = server.clone();

                let handle = std::thread::spawn(move || {
                    while completed_lookups.lock().unwrap().load(Ordering::SeqCst) < total_lookups {
                        completed_lookups
                            .lock()
                            .unwrap()
                            .fetch_add(1, Ordering::SeqCst);
                        let rtt = dns_lookup(&server, &host);
                        let mut results = results.lock().unwrap();
                        results.push(rtt);
                    }
                });

                handles.push(handle);
            }

            for handle in handles {
                handle.join().unwrap();
            }
        }
        let results = results.lock().unwrap();
        let mut sum: f64 = 0.0;
        for n in results.iter() {
            sum += *n as f64;
        }

        end_result.push(BenchmarkResult {
            min: *results.iter().min().unwrap(),
            max: *results.iter().max().unwrap(),
            avg: sum / results.len() as f64,
        });
    }
    print_progress_bar(servers.len(), servers.len());
    println!();
}
