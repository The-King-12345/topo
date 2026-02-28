use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "topo", about = "A network topology CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Show,
}