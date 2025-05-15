use bincode::{Encode, Decode};
use k256::ecdsa::{Signature, SigningKey, signature::Signer, signature::Verifier};
use sha2::{Digest, Sha256};
use crate::utils;

#[derive(Encode, Decode, Clone)]
struct TxInput {
    pub txid: String,
    pub output: u16,
    pub signature: String,
}

impl From<&TxInput> for TxInputForSign {
    fn from(input: &TxInput) -> Self {
        TxInputForSign {
            txid: input.txid.clone(),
            output: input.output,
        }
    }
}

#[derive(Encode, Decode)]
pub struct TxInputForSign {
    pub txid: String,
    pub output: u16,
}


//todo: use public key hash and include pubkey in inputs
#[derive(Encode, Decode, Clone)]
struct TxOutput {
    pub value: u64,
    pub receiver_pk: String,
}

struct Transaction {
    pub id: String,
    pub timestamp: u64,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
}

#[derive(Encode, Decode, Clone)]
struct TransactionNoID {
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub timestamp: u64,
}

#[derive(Encode, Decode)]
struct TransactionForSign {
    pub timestamp: u64,
    pub inputs: Vec<TxInputForSign>,
    pub outputs: Vec<TxOutput>,
}

impl From<&Transaction> for TransactionForSign {
    fn from(tx: &Transaction) -> Self {
        TransactionForSign {
            timestamp: tx.timestamp,
            inputs: tx.inputs.iter().map(|i| i.into()).collect(),
            outputs: tx.outputs.clone(),
        }
    }
}

impl Transaction {
    pub fn new(inputs: Vec<TxInput>, outputs: Vec<TxOutput>) -> Transaction {
        Transaction {
            id: String::new(),
            timestamp: utils::unix_timestamp(),
            inputs,
            outputs,
        }
    }

    fn as_bincode_for_sign(&self) -> Vec<u8> {
        bincode::encode_to_vec(
            &TransactionForSign::from(self),
            bincode::config::standard(),
        )
        .unwrap()
    }

    fn as_bincode_no_id(&self) -> Vec<u8> {
        let no_id = TransactionNoID {
            inputs: self.inputs.clone(),
            outputs: self.outputs.clone(),
            timestamp: self.timestamp,
        };

        bincode::encode_to_vec(no_id, bincode::config::standard()).unwrap()
    }

    fn sighash(&self) -> String {
        let data = self.as_bincode_for_sign();
        let mut hasher = Sha256::new();
        hasher.update(&data);
        format!("{:x}", hasher.finalize())
    }

    fn hash(&self) -> String {
        let data = self.as_bincode_no_id();
        let mut hasher = Sha256::new();
        hasher.update(&data);
        format!("{:x}", hasher.finalize())
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_transaction() {
        let inputs = vec![
            TxInput {
                txid: "txid1".to_string(),
                output: 0,
                signature: "signature1".to_string(),
            },
            TxInput {
                txid: "txid2".to_string(),
                output: 1,
                signature: "signature2".to_string(),
            },
        ];

        let outputs = vec![
            TxOutput {
                value: 100,
                receiver_pk: "receiver_pk1".to_string(),
            },
            TxOutput {
                value: 200,
                receiver_pk: "receiver_pk2".to_string(),
            },
        ];

        let transaction = Transaction::new(inputs, outputs);
        assert_eq!(transaction.inputs.len(), 2);
        assert_eq!(transaction.outputs.len(), 2);
    }
}
