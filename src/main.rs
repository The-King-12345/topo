mod cli;
mod network;
mod ui;

use clap::Parser;
use cli::{Cli, Commands};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Show => {
            let my_network = network::Network::load();
            ui::draw_ui(&my_network)?;
        }
        Commands::Add { address, host }=> {
            network::Network::add(address, host)?;
        }
    }
    
    Ok(())
}