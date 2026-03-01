use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::process::Command;

const FILENAME: &str = "network.toml";

#[derive(Default, Debug)]
pub struct Network {
    pub hosts: HashMap<String, HostData>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HostData {
    pub host: String, 
    
    #[serde(default)]
    pub x: f64,
    #[serde(default)]
    pub y: f64,

}

impl Network {

    pub fn load() -> Self {
        let contents = match fs::read_to_string(FILENAME) {
            Ok(c) => c,
            Err(_) => return Network::default(),
        };

        match toml::from_str::<HashMap<String, HostData>>(&contents) {
            Ok(hosts) => Network { hosts },
            Err(e) => {
                eprintln!("Warning: Failed to parse {}. Error: {}", FILENAME, e);
                Network::default()
            }
        }
    }

    pub fn add(address: String, host: String) -> io::Result<()> {
        // check if address exists
        if Network::load().hosts.contains_key(&address) { // optimize
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("entry with address '{}' already exists", address),
            ));
        }
        
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(FILENAME)?;

        if file.metadata()?.len() > 0 {
            writeln!(file)?;
        }

        let content = format!("[\'{}\']\nhost = \'{}\'", address, host);

        writeln!(file, "\n{}", content)?;
        println!("Writing to {}", FILENAME);
        println!("{}", content);
        // writeln!(file, "x = 0")?;
        // writeln!(file, "y = 0")?;
        
        Ok(())
    }

    pub fn scan_and_fill(&self, target_range: String) -> io::Result<()> {
        println!("Scanning range: {} with RustScan...", target_range);

        let output = Command::new("rustscan")
            .args([
                "-a", &target_range,
                "-p", "22,3389",
                "-g",
            ])
            .output()?;

        // Debug: print everything rustscan returned
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        println!("=== RUSTSCAN STDOUT ===\n{}", stdout);
        println!("=== RUSTSCAN STDERR ===\n{}", stderr);
        println!("=== EXIT CODE: {:?} ===", output.status.code());

        for line in stdout.lines() {
            if let Some((ip, ports_part)) = line.split_once(" -> ") {
                if !ports_part.trim_matches(|c| c == '[' || c == ']').is_empty() {
                    let hostname = format!(
                        "node-{}",
                        ip.split('.').last().unwrap_or("discovered")
                    );
                    if let Err(e) = Self::add(ip.to_string(), hostname.clone()) {
                        if e.kind() != io::ErrorKind::AlreadyExists {
                            eprintln!("Failed to add {}: {}", ip, e);
                        }
                    } else {
                        println!("Discovered and added: {} ({})", ip, hostname);
                    }
                }
            }
        }

        Ok(())
    }

}