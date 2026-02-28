use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

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
        let filename = "network.toml";

        let contents = match fs::read_to_string(filename) {
            Ok(c) => c,
            Err(_) => return Network::default(),
        };

        match toml::from_str::<HashMap<String, HostData>>(&contents) {
            Ok(hosts) => Network { hosts },
            Err(e) => {
                eprintln!("Warning: Failed to parse {}. Error: {}", filename, e);
                Network::default()
            }
        }
    }
}