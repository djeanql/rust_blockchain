use crate::transaction::{TxOutput, TxInput};
use std::collections::HashMap;
use crate::block::Block;

pub struct UTXOSet {
    utxos: HashMap<([u8; 32], u16), TxOutput>,
}

impl UTXOSet {
    pub fn new() -> UTXOSet {
        UTXOSet {
            utxos: HashMap::new(),
        }
    }

    pub fn add_utxo(&mut self, txid: [u8; 32], index: u16, output: TxOutput) {
        self.utxos.insert((txid, index), output);
    }

    pub fn remove_utxo(&mut self, txid: [u8; 32], index: u16) {
        self.utxos.remove(&(txid, index));
    }

    pub fn get_utxo(&self, txid: [u8; 32], index: u16) -> Option<&TxOutput> {
        self.utxos.get(&(txid, index))
    }

    pub fn update_with_block(&mut self, block: &Block) {
        for tx in &block.transactions {
            for input in &tx.inputs {
                self.remove_utxo(input.txid, input.output);
            }
            for (index, output) in tx.outputs.iter().enumerate() {
                self.add_utxo(tx.id, index as u16, output.clone());
            }
        }
    }
}