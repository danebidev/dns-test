use std::{
    cmp,
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
struct BenchmarkResult<'a> {
    dns: &'a DNS,
    min: u128,
    max: u128,
    avg: f64,
}

#[derive(Copy, Clone, Debug)]
enum SortType {
    Average,
    Minimum,
    Maximum,
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
    let max_concurrent_threads = std::cmp::min(10, total_lookups);
    let servers = &config::get_dns_servers().unwrap();
    let hosts = config::get_test_domains().unwrap();
    let mut end_result: Vec<BenchmarkResult> = Vec::new();

    println!("Testing default DNS servers...");

    for (i, server) in servers.into_iter().enumerate() {
        print_progress_bar(i, servers.len());

        let server_results: Arc<Mutex<Vec<u128>>> = Arc::new(Mutex::new(Vec::new()));

        let hosts_queue: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(hosts.clone()));
        let completed_lookups = Arc::new(AtomicU64::new(0));

        let mut handles = vec![];
        for _ in 0..max_concurrent_threads {
            let server = server.clone();
            let hosts_queue = Arc::clone(&hosts_queue);
            let server_results = Arc::clone(&server_results);
            let completed_lookups = Arc::clone(&completed_lookups);

            let handle = std::thread::spawn(move || loop {
                let host = {
                    let mut queue = hosts_queue.lock().unwrap();
                    if queue.is_empty() {
                        break;
                    }
                    queue.pop().unwrap()
                };

                for _ in 0..total_lookups {
                    let rtt = dns_lookup(&server, &host);

                    let mut results = server_results.lock().unwrap();
                    results.push(rtt);
                }

                completed_lookups.fetch_add(1, Ordering::SeqCst);
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let results = server_results.lock().unwrap();
        let mut sum: f64 = 0.0;

        end_result.push(BenchmarkResult {
            dns: server,
            min: *results.iter().min().unwrap_or(&0),
            max: *results.iter().max().unwrap_or(&0),
            avg: {
                for n in results.iter() {
                    sum += *n as f64;
                }
                sum / results.len() as f64
            },
        });
    }

    print_progress_bar(servers.len(), servers.len());
    println!("\n\nResults:");
    print_table(end_result);
}

fn print_section(text: &str, len: usize) {
    let spaces = len - text.len();
    print!("{}", text);
    for _ in 0..spaces {
        print!(" ")
    }
}

fn strip_port_number(ip: &str) -> &str {
    &ip[..ip.len() - 3]
}

fn print_results_line(
    result: BenchmarkResult,
    name_length: usize,
    ip_length: usize,
    avg_length: usize,
    min_length: usize,
) {
    print_section(&result.dns.name, name_length);
    print_section(strip_port_number(&result.dns.ips[0]), ip_length);
    print_section(&format!("{:.2}", result.avg), avg_length);
    print_section(&result.min.to_string(), min_length);
    print_section(&result.max.to_string(), result.max.to_string().len());

    for i in 1..result.dns.ips.len() {
        println!();
        for _ in 0..name_length {
            print!(" ");
        }
        print!("{}", strip_port_number(&result.dns.ips[i]));
    }
}

fn print_table(mut results: Vec<BenchmarkResult>) {
    // At least 2 spaces if it's from the title,
    // 1 space if from the field
    let mut name_length = "Provider".len() + 2;
    let mut ip_length = "0.0.0.0".len() + 1;
    let mut avg_length = "Avg (us)".len() + 2;
    let mut min_length = "Min".len() + 2;
    let mut max_length = "Max".len();
    for result in &results {
        name_length = cmp::max(name_length, result.dns.name.len() + 1);
        let avg_name = format!("{:.2}", result.avg);
        avg_length = cmp::max(avg_length, avg_name.len() + 1);
        min_length = cmp::max(min_length, result.min.to_string().len() + 1);
        max_length = cmp::max(max_length, result.max.to_string().len());
        for ip in &result.dns.ips {
            ip_length = cmp::max(ip_length, ip.len() + 1);
        }
    }

    print_section("Provider", name_length);
    print_section("IP", ip_length);
    print_section("Avg (μs)", avg_length + 1); // + 1 to account for the non-ascii character
    print_section("Min", min_length);
    print_section("Max\n", "Max\n".len());

    for _ in 0..name_length + ip_length + avg_length + min_length + max_length {
        print!("-");
    }
    println!();

    match config::get_sort_type().unwrap() {
        SortType::Average => {
            results.sort_by(|a: &BenchmarkResult, b: &BenchmarkResult| a.avg.total_cmp(&b.avg))
        }
        SortType::Minimum => {
            results.sort_by(|a: &BenchmarkResult, b: &BenchmarkResult| a.min.cmp(&b.min))
        }
        SortType::Maximum => {
            results.sort_by(|a: &BenchmarkResult, b: &BenchmarkResult| a.max.cmp(&b.max))
        }
    }

    for result in results {
        print_results_line(result, name_length, ip_length, avg_length, min_length);
        println!();
    }
}
