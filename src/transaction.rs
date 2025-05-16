use crate::utils;
use bincode::{Decode, Encode};
use k256::ecdsa::signature::Verifier;
use k256::ecdsa::{Signature, SigningKey, signature::Signer};
use sha2::{Digest, Sha256};
use std::fmt;

//TODO: use ed25519
//TODO: use references instead of copying

#[derive(Encode, Decode, Clone)]
pub struct TxInput {
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

    pub fn verify_signature(&self) -> Result<(), TransactionError> {
        let tx_for_sign: TxInputForSign = self.clone().into();

        let verify_key =
            k256::ecdsa::VerifyingKey::from_sec1_bytes(&self.pubkey).map_err(|_| TransactionError::InvalidPublicKey)?;

        let signature =
            k256::ecdsa::Signature::from_bytes((&self.signature).into()).map_err(|_| TransactionError::InvalidSignature)?;

        verify_key
            .verify(&tx_for_sign.sighash(), &signature)
            .map_err(|_| TransactionError::SignatureVerificationFailed)?;

        Ok(())
    }
}

impl TxInput {
    pub fn new_unsigned(txid: [u8; 32], output: u16, pubkey: [u8; 33]) -> TxInput {
        TxInput {
            txid,
            output,
            signature: [0; 64],
            pubkey,
        }
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

#[derive(Encode, Decode, Clone)]
pub struct TxOutput {
    pub value: u64,
    pub pkhash: [u8; 32],
}

impl TxOutput {
    pub fn new(value: u64, pkhash: [u8; 32]) -> TxOutput {
        TxOutput { value, pkhash }
    }
}

#[derive(Debug)]
pub enum TransactionError {
    InvalidPublicKey,
    InvalidSignature,
    SignatureVerificationFailed,
    InvalidID,
    InvalidTimestamp,
    ZeroValueOutput,
    DuplicateInput,
    DuplicateOutput,
}

#[derive(Encode, Decode, Clone)]
struct TransactionNoID {
    inputs: Vec<TxInput>,
    outputs: Vec<TxOutput>,
    pub timestamp: u64,
}

#[derive(Encode, Decode)]
pub struct Transaction {
    pub id: [u8; 32],
    pub timestamp: u64,
    inputs: Vec<TxInput>,
    outputs: Vec<TxOutput>,
}

impl Transaction {
    pub fn new(inputs: Vec<TxInput>, outputs: Vec<TxOutput>) -> Transaction {
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

    fn verify_signatures(&self) -> Result<(), TransactionError> {
        for input in &self.inputs {
            input.verify_signature()?;
        }
        Ok(())
    }

    pub fn verify(&self) -> Result<(), TransactionError> {
        self.verify_signatures()?;
        
        if self.id != self.hash() {
            return Err(TransactionError::InvalidID);
        } else if self.timestamp > utils::unix_timestamp() {
            return Err(TransactionError::InvalidTimestamp);
        }
        
        for input in &self.inputs {
            if self.inputs.iter().filter(|i| i.txid == input.txid && i.output == input.output).count() > 1 {
                return Err(TransactionError::DuplicateInput);
            }
        }

        for output in &self.outputs {
            if output.value == 0 {
                return Err(TransactionError::ZeroValueOutput);
            }
            if self.outputs.iter().filter(|o| o.pkhash == output.pkhash).count() > 1 {
                return Err(TransactionError::DuplicateOutput);
            }
        }
        Ok(())
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Transaction ID: {}", hex::encode(self.id))?;
        writeln!(f, "Timestamp: {}", self.timestamp)?;
        writeln!(f, "Inputs:")?;
        for input in &self.inputs {
            writeln!(f, "  TxID: {}, Output: {}, Signature: {}, Pubkey: {}",
                hex::encode(input.txid),
                input.output,
                hex::encode(input.signature),
                hex::encode(input.pubkey))?;
        }
        writeln!(f, "Outputs:")?;
        for output in &self.outputs {
            writeln!(f, "  Value: {}, PKHash: {}",
                output.value,
                hex::encode(output.pkhash))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallet::Wallet;

    #[test]
    fn test_create_blank_transaction() {
        let inputs = vec![
            TxInput::new_unsigned([0; 32], 2, [0; 33]),
            TxInput::new_unsigned([0; 32], 1, [0; 33]),
        ];

        let outputs = vec![
            TxOutput::new(100, [0; 32]),
            TxOutput::new(200, [0; 32]),
        ];

        let transaction = Transaction::new(inputs, outputs);
        println!("{}", transaction);
        assert_eq!(transaction.inputs.len(), 2);
        assert_eq!(transaction.outputs.len(), 2);
    }

    #[test]
    fn test_sign() {
        let wallet = Wallet::new();

        let inputs = vec![
            TxInput::new_unsigned([0; 32], 2, [0; 33]),
            TxInput::new_unsigned([0; 32], 1, [0; 33]),
        ];

        let outputs = vec![
            TxOutput::new(100, [0; 32]),
            TxOutput::new(200, [1; 32]),
        ];

        let mut transaction = Transaction::new(inputs, outputs);

        wallet.sign_transaction(&mut transaction);

        assert!(!transaction.id.is_empty());
        assert!(!transaction.inputs[0].signature.is_empty());
        assert!(transaction.inputs[0].signature != transaction.inputs[1].signature);
        assert!(transaction.id == transaction.hash());

        assert!(transaction.verify().is_ok());
    }

    #[test]
    fn test_sign_invalid() {
        let wallet = Wallet::new();

        let inputs = vec![
            TxInput::new_unsigned([0; 32], 2, [0; 33]),
            TxInput::new_unsigned([0; 32], 1, [0; 33]),
        ];

        let outputs = vec![
            TxOutput::new(100, [0; 32]),
            TxOutput::new(200, [0; 32]),
        ];

        let mut transaction = Transaction::new(inputs, outputs);

        wallet.sign_transaction(&mut transaction);

        transaction.inputs[0].signature[0] = 1;

        assert!(matches!(transaction.verify(), Err(TransactionError::SignatureVerificationFailed)));
    }

    #[test]
    fn test_fails_if_signature_tampered() {
        let mut tx = Transaction::new(
            vec![TxInput::new_unsigned([0;32], 0, [0;33])],
            vec![TxOutput::new(50, [0;32])]
        );
        let wallet = Wallet::new();
        wallet.sign_transaction(&mut tx);
        assert!(tx.verify().is_ok());

        // tamper
        tx.inputs[0].signature[0] ^= 0xFF;
        assert!(matches!(tx.verify(), Err(TransactionError::SignatureVerificationFailed)));
    }

    #[test]
    fn test_fails_if_pubkey_tampered() {
        let mut tx = Transaction::new(
            vec![TxInput::new_unsigned([0;32], 0, [0;33])],
            vec![TxOutput::new(50, [0;32])]
        );
        let wallet = Wallet::new();
        wallet.sign_transaction(&mut tx);
        assert!(tx.verify().is_ok());

        // tamper
        tx.inputs[0].pubkey[1] ^= 0xAA;
        let result = tx.verify();
        assert!(matches!(result, Err(TransactionError::SignatureVerificationFailed)) ||
                matches!(result, Err(TransactionError::InvalidPublicKey)));
    }

    #[test]
    fn test_fails_if_input_data_tampered() {
        let mut tx = Transaction::new(
            vec![TxInput::new_unsigned([1;32], 2, [0;33])],
            vec![TxOutput::new(50, [0;32])]
        );
        let wallet = Wallet::new();
        wallet.sign_transaction(&mut tx);
        assert!(tx.verify().is_ok());

        // tamper the TxInput’s `output` index
        tx.inputs[0].output = 3;
        assert!(matches!(tx.verify(), Err(TransactionError::SignatureVerificationFailed)));
    }

}
