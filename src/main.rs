use clap::Parser;
use libbitcoinkernel_sys::{
    BlockManagerOptions, ChainstateLoadOptions, ChainstateManager, ChainstateManagerOptions, ChainType,
};

mod scanner;
mod kernel;

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
    let chain_type = match args.network.to_lowercase().as_str() {
        "mainnet" => ChainType::MAINNET,
        "testnet" => ChainType::TESTNET,
        "regtest" => ChainType::REGTEST,
        "signet" => ChainType::SIGNET,
        _ => {
            eprintln!("Invalid network type: {}", args.network);
            std::process::exit(1);
        }
    };
    let data_dir = args.datadir;
    let blocks_dir = data_dir.clone() + "/blocks";

    // Set up the silent payment keys
    let (receiver, secret_scan_key) = scanner::parse_keys(args.scankey, args.spendpubkey);

    // Set up the kernel
    let _ = kernel::setup_logging().unwrap();
    let context = kernel::create_context(chain_type);
    let chainman = ChainstateManager::new(
        ChainstateManagerOptions::new(&context, &data_dir).unwrap(),
        BlockManagerOptions::new(&context, &blocks_dir).unwrap(),
        &context,
    )
    .unwrap();
    chainman
        .load_chainstate(ChainstateLoadOptions::new())
        .unwrap();
    chainman.import_blocks().unwrap();
    scanner::scan_txs(&chainman, &receiver, &secret_scan_key);
}
