use rust_blockchain::blockchain::Blockchain;
use rust_blockchain::transaction::{Transaction, TxInput, TxOutput};
use rust_blockchain::utils::*;
use rust_blockchain::wallet::Wallet;

fn main() {
    println!("Hello, world!");

    let wallet = Wallet::new();
    let mut blockchain = Blockchain::new();

    for _ in 0..5 {
        let mut block = blockchain.next_block();
        println!("Mining block {}...", block.index);
        mine(&mut block, wallet.pkhash, blockchain.get_block_reward());
        blockchain.add_block(block).unwrap();
    }

    println!("\nUTXO SET:\n{}\n", blockchain.utxos);

    let (txid, index) = &blockchain.utxos.utxos_from_pkhash(wallet.pkhash)[0];

    let mut block = blockchain.next_block();

    let inputs = vec![
        TxInput::new_unsigned(*txid, *index), // use coinbase UTXO
    ];

    let outputs = vec![
        TxOutput::new(100, [0; 32]),        // unspendable
        TxOutput::new(2000, wallet.pkhash), // send to self
        TxOutput::new(42, [1; 32]),
    ];

    let mut tx = Transaction::new(inputs, outputs);
    wallet.sign_transaction(&mut tx);

    block.add_tx(tx);
    mine(&mut block, wallet.pkhash, blockchain.get_block_reward());
    blockchain.add_block(block).unwrap();

    println!("{}", blockchain);

    println!("\nNEW UTXO SET:\n{}\n", blockchain.utxos);
}
