use once_cell::sync::Lazy;
use std::{path::PathBuf, sync::Mutex};

struct Config {
    config_file: PathBuf 
}

static CONFIG: Lazy<Mutex<Config>> = Lazy::new(||  Mutex::new(Config {
    config_file: PathBuf::from("")
}));

fn update_path(path: PathBuf) {
    let mut cfg = CONFIG.lock().unwrap();
    cfg.config_file = path;
}

fn get_path() -> PathBuf {
    CONFIG.lock().unwrap().config_file.clone()
}

fn main() {
}
