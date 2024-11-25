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
    Min,
    Max,
    Lost,
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let mut config_path: PathBuf;
    match std::env::var("XDG_CONFIG_HOME") {
        Ok(s) => config_path = PathBuf::from(s),
        _ => config_path = PathBuf::from(shellexpand::tilde("~/.config").to_string()),
    };

    config_path.push("dnstest-cli/config.toml");

    let sort: SortType = SortType::Average;

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
                    if i + 1 >= args.len() {
                        println!("Missing config file path after '--config'");
                        return;
                    } else {
                        i += 1;
                        config_path = PathBuf::from(shellexpand::tilde(&args[i]).to_string());
                    }
                }
                "sort" => {
                    // TODO: fix ugly code repetition
                    if i + 1 >= args.len() {
                        println!("Missing sort type after '--sort'");
                        return;
                    } else {
                        i += 1;
                        // TODO
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

    config::init(config_path);
}
