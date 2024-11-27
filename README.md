# dnstest-cli

A blazingly fast CLI tool written in Rust for benchmarking DNS server performance.

## Installation

Clone and build from source:

```bash
git clone https://github.com/danebidev</dnstest-cli
cd dnstest-cli
cargo build --release
```

The binary will be available in `target/release/dnstest-cli`

## Usage

```bash
dnstest-cli [options]
```

When run without options it will run benchmarks on a list of default DNS servers.

When first run the current DNS server list will also be written in `${XDG_CONFIG_HOME}/.config/dnstest-cli/config.toml` (or `${HOME}/.config/dnstest-cli/config.toml`, if `XDG_CONFIG_HOME` isn't set) and will be used as server list from the next time. You can then modify the file to add your own DNS servers.

Options:

- `--config`: Config file path 
- `--queries`: Number of queries (default: 100)
- `--servers`: Add custom DNS server IPs (comma-separated)
- `--sort`: Sort servers based on avg, min, max or lost% (default: avg)

## Example Output

```
Testing default DNS servers...
[====================] 100% Complete

Results:
Provider   IP             Avg (μs)   Min    Max
------------------------------------------------
Cloudflare 1.1.1.1        14.3       8.1    45.2
           1.0.0.1        
Google     8.8.8.8        16.2       9.2    51.3
           8.8.4.4        
Quad9      9.9.9.9        18.4       10.1   55.4
OpenDNS    208.67.222.222 19.1       10.8   58.2
```
