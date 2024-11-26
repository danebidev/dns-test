use std::path::PathBuf;

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
    let mut queries: i32 = 100;

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];

        if !arg.starts_with("--") {
            println!("Unrecognized option '{arg}'");
            return;
        } else {
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

                    match next_arg.parse::<i32>() {
                        Ok(parsed) => queries = parsed,
                        Err(_) => {
                            println!("Error parsing parameter to --query option '{}'", next_arg)
                        }
                    }
                }
                // Rest of the options
                _ => {
                    println!("Unrecognized option '{arg}'");
                    return;
                }
            }
        }

        i += 1;
    }

    config::init(config_path, sort, queries);
}
