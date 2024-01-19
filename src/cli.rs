use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "blockchain", version, about="a simple btc", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add block
    #[command(name = "addblock")]
    Addblock {
        /// Block data
        #[arg(short, long)]
        data: String,
    },
    /// Print block chain info
    #[command(name = "printchain")]
    PrintChain,
    /// Create block chain
    #[command(name = "createblockchain")]
    CreateBlockChain {
        /// 创世的地址
        #[arg(short, long)]
        address: String,
    },
    #[command(name = "getbalance")]
    GetBalance {
        #[arg(short, long)]
        address: String,
    },
    #[command(name = "send")]
    Send {
        #[arg(short, long)]
        from: String,
        #[arg(short, long)]
        to: String,
        #[arg(short, long)]
        amount: isize,
    },
    #[command(name = "createwallet")]
    CreateWallet,
}
