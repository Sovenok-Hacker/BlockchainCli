use blockchaintree::block::Block as _;
use blockchaintree::static_values::BLOCKS_PER_EPOCH;
use blockchaintree::tools;
use blockchaintree::{blockchaintree::BlockChainTree, static_values};
use primitive_types::U256;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn mine(tree: &mut BlockChainTree, wallet: [u8; 33], transactions: &[[u8; 32]]) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let main_chain = tree.get_main_chain();

    loop {
        println!("Current height: {}", main_chain.get_height());
        println!(
            "Current miner balance: {}",
            tree.get_amount(&wallet).unwrap()
        );
        println!(
            "Current root balance: {}",
            tree.get_amount(&static_values::ROOT_PUBLIC_ADDRESS)
                .unwrap()
        );
        let mut nonce = U256::zero();
        let last_block = main_chain.get_last_block().unwrap().unwrap();
        let prev_hash = last_block.hash().unwrap();
        let difficulty = last_block.get_info().difficulty;
        println!(
            "Current difficulty: {}",
            tools::count_leading_zeros(&difficulty)
        );
        while nonce < U256::MAX {
            let mut pow = [0u8; 32];
            nonce.to_big_endian(&mut pow);
            if tools::check_pow(&prev_hash, &difficulty, &pow) {
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                println!("Found nonce! {}", nonce);

                let transactions: &[[u8; 32]] =
                    if ((last_block.get_info().height + 1) % BLOCKS_PER_EPOCH).is_zero() {
                        println!("Cycle ended!");
                        &[]
                    } else {
                        transactions
                    };

                let block = rt
                    .block_on(tree.emmit_new_main_block(&pow, &wallet, transactions, timestamp))
                    .unwrap();

                // Node should handle this
                tree.send_amount(
                    &static_values::ROOT_PUBLIC_ADDRESS,
                    &wallet,
                    *static_values::MAIN_CHAIN_PAYMENT,
                )
                .unwrap();

                let fee = tools::recalculate_fee(&last_block.get_info().difficulty);
                for _ in transactions {
                    tree.add_amount(&wallet, fee).unwrap();
                }

                println!("Added new block! {:?}\n", block.hash().unwrap());

                rt.block_on(tree.flush()).unwrap();
                break;
            }
            nonce += U256::one();
        }
    }
}

pub fn mine_transactions(tree: &mut BlockChainTree, wallet: [u8; 33], transactions: &[[u8; 32]]) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let main_chain = tree.get_main_chain();

    println!("Current height: {}", main_chain.get_height());
    println!(
        "Current miner balance: {}",
        tree.get_amount(&wallet).unwrap()
    );
    println!(
        "Current root balance: {}",
        tree.get_amount(&static_values::ROOT_PUBLIC_ADDRESS)
            .unwrap()
    );
    let mut nonce = U256::zero();
    let last_block = main_chain.get_last_block().unwrap().unwrap();
    let prev_hash = last_block.hash().unwrap();
    let difficulty = last_block.get_info().difficulty;
    println!(
        "Current difficulty: {}",
        tools::count_leading_zeros(&difficulty)
    );
    while nonce < U256::MAX {
        let mut pow = [0u8; 32];
        nonce.to_big_endian(&mut pow);
        if tools::check_pow(&prev_hash, &difficulty, &pow) {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            println!("Found nonce! {}", nonce);

            let transactions: &[[u8; 32]] =
                if ((last_block.get_info().height + 1) % BLOCKS_PER_EPOCH).is_zero() {
                    println!("Cycle ended!");
                    &[]
                } else {
                    transactions
                };

            let block = rt
                .block_on(tree.emmit_new_main_block(&pow, &wallet, transactions, timestamp))
                .unwrap();

            // Node should handle this
            tree.send_amount(
                &static_values::ROOT_PUBLIC_ADDRESS,
                &wallet,
                *static_values::MAIN_CHAIN_PAYMENT,
            )
            .unwrap();

            let fee = tools::recalculate_fee(&last_block.get_info().difficulty);
            for _ in transactions {
                tree.add_amount(&wallet, fee).unwrap();
            }

            println!("Added new block! {:?}\n", block.hash().unwrap());

            rt.block_on(tree.flush()).unwrap();
            break;
        }
        nonce += U256::one();
    }
}

pub fn mine_derivative(tree: &mut BlockChainTree, wallet: [u8; 33]) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let chain = tree.get_derivative_chain(&wallet).unwrap();

    loop {
        println!("Current height: {}", chain.get_height());
        println!(
            "Current miner gas amount: {}",
            tree.get_gas(&wallet).unwrap()
        );
        let mut nonce = U256::zero();
        let (prev_hash, difficulty, _prev_timestamp, _height) =
            if let Some(block) = chain.get_last_block().unwrap() {
                (
                    block.hash().unwrap(),
                    block.get_info().difficulty,
                    block.get_info().timestamp,
                    block.get_info().height,
                )
            } else {
                let block = tree
                    .get_main_chain()
                    .find_by_hash(&chain.genesis_hash)
                    .unwrap()
                    .unwrap();
                (
                    block.hash().unwrap(),
                    static_values::BEGINNING_DIFFICULTY,
                    block.get_info().timestamp,
                    U256::zero(),
                )
            };
        println!(
            "Current difficulty: {}",
            tools::count_leading_zeros(&difficulty)
        );
        while nonce < U256::MAX {
            let mut pow = [0u8; 32];
            nonce.to_big_endian(&mut pow);
            if tools::check_pow(&prev_hash, &difficulty, &pow) {
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                println!("Found nonce! {}", nonce);

                let block = rt
                    .block_on(tree.emmit_new_derivative_block(&pow, &wallet, timestamp))
                    .unwrap();

                // Node should handle this
                tree.add_gas(&wallet, *static_values::MAIN_CHAIN_PAYMENT)
                    .unwrap();

                println!("Added new block! {:?}\n", block.hash().unwrap());

                rt.block_on(chain.flush()).unwrap();
                rt.block_on(tree.flush()).unwrap();
                break;
            }
            nonce += U256::one();
        }
    }
}
