use crate::transaction::TxOutput;
use std::collections::HashMap;
use crate::block::Block;
use std::fmt;


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

    pub fn get_utxos(&self) -> &HashMap<([u8; 32], u16), TxOutput> {
        &self.utxos
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

    pub fn utxos_from_pkhash(&self, pkhash: [u8; 32]) -> Vec<([u8; 32], u16)> {
        self.utxos
            .iter()
            .filter_map(|((txid, index), output)| {
                if output.pkhash == pkhash {
                    Some((*txid, *index))
                } else {
                    None
                }
            })
            .collect()
    }
}

impl fmt::Display for UTXOSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for ((txid, index), output) in &self.utxos {
            write!(f, "TxID: {}, Index: {}, Output: {}", hex::encode(txid), index, output)?;
        }
        Ok(())
    }
}
