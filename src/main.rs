use std::time::{SystemTime, UNIX_EPOCH};

use blockchaintree::{
    blockchaintree::BlockChainTree,
    transaction::{Transaction, Transactionable},
};
use clap::{Parser, Subcommand};

use primitive_types::U256;
use secp256k1::{rand, Secp256k1, SecretKey};

mod mine;

use mine::{mine, mine_derivative, mine_transactions};

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

    /// Mine some gas and coind on the derivative chain
    SendTransaction {
        private_key: String,
        to: String,
        amount: u64,
    },

    /// Get transaction by the hash
    Transaction { hash: String },

    /// Get block by height
    BlockId { height: usize },
}

fn main() {
    let cli = Args::parse();

    let mut blockchain: Option<BlockChainTree> = None;

    match &cli.command {
        Commands::Keygen => {
            let mut rand = rand::thread_rng();
            let secp = Secp256k1::new();
            let (secret_key, public_key) = secp.generate_keypair(&mut rand);

            println!("Address: 0x{}", public_key);
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
                mine(blockchain, address_bytes, &[[25; 32]]);
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
        Commands::SendTransaction {
            private_key,
            to,
            amount,
        } => {
            let mut address_bytes = [0; 33];

            hex::decode_to_slice(to.trim_start_matches("0x"), &mut address_bytes)
                .expect("A public key of lenght 33 bytes expected");

            let mut private_key_bytes = [0; 32];

            hex::decode_to_slice(private_key.trim_start_matches("0x"), &mut private_key_bytes)
                .expect("A private key of lenght 32 bytes expected");
            let context = secp256k1::Secp256k1::new();
            let secret_key_serialized = SecretKey::from_slice(&private_key_bytes).unwrap();
            let sender_address = secret_key_serialized.public_key(&context).serialize();

            if let Some(blockchain) = blockchain.as_mut() {
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let transaction = blockchaintree::transaction::Transaction::new(
                    sender_address,
                    address_bytes,
                    timestamp,
                    U256::from(*amount as usize),
                    private_key_bytes,
                    None,
                )
                .expect("Error building transaction");
                blockchain
                    .send_transaction(&transaction)
                    .expect("Error sending transaction");

                println!("Transaction 0x{} was sent", hex::encode(transaction.hash()));
                mine_transactions(blockchain, address_bytes, &[transaction.hash()]);
                println!("Block mined");
            }
        }
        Commands::Transaction { hash } => {
            let mut hash_bytes = [0; 32];

            hex::decode_to_slice(hash.trim_start_matches("0x"), &mut hash_bytes)
                .expect("A transaction hash of length 32 bytes expected");

            if let Some(blockchain) = blockchain {
                let tx: Transaction = blockchain
                    .get_main_chain()
                    .get_transaction(&hash_bytes)
                    .expect("Error getting transaction")
                    .expect("Transaction not found");
                println!(
                    "{}: 0x{} -> 0x{} - {} aplo",
                    hash,
                    hex::encode(tx.get_sender()),
                    hex::encode(tx.get_receiver()),
                    tx.get_amount()
                );
            }
        }
        Commands::BlockId { height } => {
            if let Some(blockchain) = blockchain {
                let main_chain = blockchain.get_main_chain();

                let block = main_chain
                    .find_by_height(&U256::from(*height))
                    .expect("Unable to find block")
                    .expect("Unable to find block");

                println!("Hash: 0x{}", hex::encode(block.hash().unwrap()));
                println!("Basic info: {:?}", block.get_info());
                println!(
                    "Transactions: {:#?}",
                    block.transactions().map(|trxs| trxs
                        .iter()
                        .map(hex::encode)
                        .collect::<Vec<_>>())
                );
            }
        }
    }
}
