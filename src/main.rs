use clap::Parser;

/// A simple CLI tool
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Data directory
    #[arg(long)]
    datadir: String,

    /// Network
    #[arg(long)]
    network: String,

    /// Scan key
    #[arg(long)]
    scankey: String,

    /// Spend public key
    #[arg(long)]
    spendpubkey: String,
}

fn main() {
    let args = Args::parse();

    println!("Data directory: {}", args.datadir);
    println!("Network: {}", args.network);
    println!("Scan key: {}", args.scankey);
    println!("Spend public key: {}", args.spendpubkey);
}
