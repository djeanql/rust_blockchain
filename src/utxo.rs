use crate::utils;
use bincode::{Decode, Encode};
use k256::ecdsa::signature::Verifier;
use k256::ecdsa::{Signature, SigningKey, signature::Signer};
use sha2::{Digest, Sha256};

//TODO: use ed25519
//TODO: use bytes instead of strings

#[derive(Encode, Decode, Clone)]
struct TxInput {
    pub txid: [u8; 32],
    pub output: u16,
    pub signature: [u8; 64],
    pub pubkey: [u8; 33],
}

impl TxInput {
    pub fn sign(&mut self, signing_key: &SigningKey) {
        self.pubkey = signing_key.verifying_key().to_encoded_point(true).as_bytes().try_into().unwrap();

        let tx_for_sign: TxInputForSign = self.clone().into();
        
        let signature: Signature = signing_key.sign(&tx_for_sign.sighash());
        self.signature = signature.to_bytes().into();
    }

    pub fn verify_signature(&self) -> bool {
        let tx_for_sign: TxInputForSign = self.clone().into();

        let verify_key =
            k256::ecdsa::VerifyingKey::from_sec1_bytes(&self.pubkey).expect("Invalid public key");

        let signature =
            k256::ecdsa::Signature::from_bytes(&self.signature.into()).expect("Invalid signature");

        verify_key
            .verify(&tx_for_sign.sighash(), &signature)
            .is_ok()
    }
}

#[derive(Encode, Decode, Debug)]
pub struct TxInputForSign {
    pub txid: [u8; 32],
    pub output: u16,
    pub pubkey: [u8; 33],
}

impl TxInputForSign {
    fn as_bincode(&self) -> Vec<u8> {
        bincode::encode_to_vec(self, bincode::config::standard()).unwrap()
    }

    fn sighash(&self) -> [u8; 32] {
        let data = self.as_bincode();
        Sha256::digest(&data).to_vec().try_into().unwrap()
    }
}

impl From<TxInput> for TxInputForSign {
    fn from(input: TxInput) -> Self {
        TxInputForSign {
            txid: input.txid,
            output: input.output,
            pubkey: input.pubkey,
        }
    }
}

//TODO: use public key hash
#[derive(Encode, Decode, Clone)]
struct TxOutput {
    pub value: u64,
    pub pkhash: [u8; 32],
}

pub struct Transaction {
    pub id: [u8; 32],
    pub timestamp: u64,
    inputs: Vec<TxInput>,
    outputs: Vec<TxOutput>,
}

#[derive(Encode, Decode, Clone)]
struct TransactionNoID {
    inputs: Vec<TxInput>,
    outputs: Vec<TxOutput>,
    pub timestamp: u64,
}

impl Transaction {
    fn new(inputs: Vec<TxInput>, outputs: Vec<TxOutput>) -> Transaction {
        Transaction {
            id: [0; 32],
            timestamp: utils::unix_timestamp(),
            inputs,
            outputs,
        }
    }

    fn as_bincode_no_id(&self) -> Vec<u8> {
        let no_id = TransactionNoID {
            inputs: self.inputs.clone(),
            outputs: self.outputs.clone(),
            timestamp: self.timestamp,
        };

        bincode::encode_to_vec(no_id, bincode::config::standard()).unwrap()
    }

    fn hash(&self) -> [u8; 32] {
        let data = self.as_bincode_no_id();
        Sha256::digest(&data).to_vec().try_into().unwrap()
    }

    pub fn sign(&mut self, signing_key: &SigningKey) {
        self.inputs.iter_mut().for_each(|input| {
            input.sign(signing_key);
        });
        self.id = self.hash();
    }

    pub fn verify_signatures(&self) -> bool {
        self.inputs.iter().all(|input| input.verify_signature())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallet::Wallet;

    #[test]
    fn test_create_blank_transaction() {
        let inputs = vec![
            TxInput {
                txid: [0; 32],
                output: 0,
                signature: [0; 64],
                pubkey: [0; 33],
            },
            TxInput {
                txid: [1; 32],
                output: 1,
                signature: [0; 64],
                pubkey: [0; 33],
            },
        ];

        let outputs = vec![
            TxOutput {
                value: 100,
                pkhash: [0; 32],
            },
            TxOutput {
                value: 200,
                pkhash: [0; 32],
            },
        ];

        let transaction = Transaction::new(inputs, outputs);
        assert_eq!(transaction.inputs.len(), 2);
        assert_eq!(transaction.outputs.len(), 2);
    }

    #[test]
    fn test_sign() {
        let wallet = Wallet::new();

        let inputs = vec![
            TxInput {
                txid: [0; 32],
                output: 0,
                signature: [0; 64],
                pubkey: [0; 33],
            },
            TxInput {
                txid: [0; 32],
                output: 1,
                signature: [0; 64],
                pubkey: [0; 33],
            },
        ];

        let outputs = vec![
            TxOutput {
                value: 100,
                pkhash: wallet.pkhash,
            },
            TxOutput {
                value: 200,
                pkhash: wallet.pkhash,
            },
        ];

        let mut transaction = Transaction::new(inputs, outputs);

        wallet.sign_utxo_based_transaction(&mut transaction);

        assert!(!transaction.id.is_empty());
        assert!(!transaction.inputs[0].signature.is_empty());
        assert!(transaction.inputs[0].signature != transaction.inputs[1].signature);
        assert!(transaction.id == transaction.hash());

        assert!(transaction.verify_signatures());
    }
}
