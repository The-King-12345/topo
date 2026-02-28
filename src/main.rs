use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    ip: String,
}

fn main() {
    let args = Args::parse();
    println!("IP: {}", args.ip);
}
