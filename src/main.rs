use blockchaintree::blockchaintree::BlockChainTree;
use clap::{Parser, Subcommand};

use secp256k1::{rand, Secp256k1};

mod mine;

use mine::{mine, mine_derivative};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
enum Commands {
    /// Generates public and private keys
    Keygen,

    /// Gets user balance
    Balance { address: String },

    /// Gets user gas
    Gas { address: String },

    /// Mine some coins on the main chain
    Mine { address: String },

    /// Mine some gas and coind on the derivative chain
    MineDerivative { address: String },
}

fn main() {
    let cli = Args::parse();

    let mut blockchain: Option<BlockChainTree> = None;

    match &cli.command {
        Commands::Keygen => {
            let mut rand = rand::thread_rng();
            let secp = Secp256k1::new();
            let (secret_key, public_key) = secp.generate_keypair(&mut rand);

            println!("Address: 0x{}", public_key.to_string());
            println!(
                "Private key (KEEP IT AS SECRET): 0x{}",
                secret_key.display_secret()
            );
            return;
        }
        _ => {
            blockchain.replace(BlockChainTree::new("./BlockChainTree").unwrap());
        }
    }

    match &cli.command {
        Commands::Keygen => unreachable!(),
        Commands::Balance { address } => {
            let mut address_bytes = [0; 33];

            hex::decode_to_slice(address.trim_start_matches("0x"), &mut address_bytes)
                .expect("A public key of lenght 33 bytes expected");

            if let Some(blockchain) = blockchain {
                let balance = blockchain
                    .get_amount(&address_bytes)
                    .expect("Getting amount failed");

                println!("Balance of address `{}` is {} aplo", address, balance);
            }
        }
        Commands::Mine { address } => {
            let mut address_bytes = [0; 33];

            hex::decode_to_slice(address.trim_start_matches("0x"), &mut address_bytes)
                .expect("A public key of lenght 33 bytes expected");

            if let Some(blockchain) = blockchain.as_mut() {
                mine(blockchain, address_bytes);
            }
        }
        Commands::MineDerivative { address } => {
            let mut address_bytes = [0; 33];

            hex::decode_to_slice(address.trim_start_matches("0x"), &mut address_bytes)
                .expect("A public key of lenght 33 bytes expected");

            if let Some(blockchain) = blockchain.as_mut() {
                mine_derivative(blockchain, address_bytes);
            }
        }
        Commands::Gas { address } => {
            let mut address_bytes = [0; 33];

            hex::decode_to_slice(address.trim_start_matches("0x"), &mut address_bytes)
                .expect("A public key of lenght 33 bytes expected");

            if let Some(blockchain) = blockchain {
                let balance = blockchain
                    .get_gas(&address_bytes)
                    .expect("Getting gas failed");

                println!(
                    "Available gas of address `{}` is {} gaplo",
                    address, balance
                );
            }
        }
    }
}
