mod cli;
mod ui;

use clap::Parser;
use cli::{Cli, Commands};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Show => {
            ui::draw_ui()?;
        }
    }

    Ok(())
}