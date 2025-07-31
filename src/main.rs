use std::error::Error;
use std::process::Command;

const DROP_URL: &str = "https://www.spamhaus.org/drop/drop.txt";
const EDROP_URL: &str = "https://www.spamhaus.org/drop/edrop.txt";
const CHAIN_NAME: &str = "SPAMHAUS_DROP";

fn run_iptables(args: &[&str]) {
    let output = Command::new("iptables")
        .args(args)
        .output()
        .expect("Failed to run iptables");

    if !output.status.success() {
        eprintln!(
            "iptables error: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

fn ensure_chain_exists() {
    let _ = Command::new("iptables").args(["-N", CHAIN_NAME]).output();
    run_iptables(&["-D", "INPUT", "-j", CHAIN_NAME]);
    run_iptables(&["-I", "INPUT", "-j", CHAIN_NAME]);
}

fn add_ip(ip: &str) {
    run_iptables(&["-A", CHAIN_NAME, "-s", ip, "-j", "DROP"]);
}

fn fetch_list(url: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let body = reqwest::blocking::get(url)?.text()?;
    let ips = body
        .lines()
        .filter(|line| !line.starts_with(";") && !line.trim().is_empty())
        .map(|line| line.split(';').next().unwrap().trim().to_string())
        .collect();
    Ok(ips)
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("[*] Fetching Spamhaus DROP lists...");

    let mut all_ips = fetch_list(DROP_URL)?;
    all_ips.extend(fetch_list(EDROP_URL)?);

    println!("[*] Retrieved {} IP ranges", all_ips.len());

    ensure_chain_exists();

    for ip in all_ips {
        add_ip(&ip);
    }

    println!("[+] iptables updated with Spamhaus ranges.");
    Ok(())
}
