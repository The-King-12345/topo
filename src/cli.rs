use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "topo", about = "A network topology CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Display the network diagram")]
    Show,
    #[command(about = "Add a host entry to network.toml")]
    Add {
        #[arg(value_name = "address")]
        address: String,
        #[arg(value_name = "hostname")]
        host: String,
    },
    #[command(about = "Scan the network using rustscan to discover hosts")]
    Scan {
        #[arg(value_name = "target", help = "The IP range or CIDR to scan (e.g. 192.168.1.0/24)")]
        target: String,
    }
}