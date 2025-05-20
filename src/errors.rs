#[derive(Debug, PartialEq)]
pub enum BlockValidationError {
    InvalidPreviousHash,
    InvalidIndex,
    InvalidTimestamp,
    InvalidProofOfWork,
    HashDigestMismatch,
    TimestampInFuture,
    InvalidTransactions(TransactionError),
}

#[derive(Debug, PartialEq)]
pub enum TransactionError {
    InvalidPublicKey,
    InvalidSignature,
    SignatureVerificationFailed,
    InvalidID,
    InvalidTimestamp,
    ZeroValueOutput,
    DuplicateInput,
    DuplicateOutput,
    EmptyInputs,
    EmptyOutputs,
    InvalidCoinbase,
    // stateful validation errors
    InvalidUTXO,
    Overspend,
    UnauthorizedSpend,
}


