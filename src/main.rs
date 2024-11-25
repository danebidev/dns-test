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
            ips: self.ips.clone()
        }
    }
}

fn main() {
    config::init();

}
