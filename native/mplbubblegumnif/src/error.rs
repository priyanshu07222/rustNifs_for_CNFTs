// use rustler::{Error, Term};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NifError {
    #[error("Invalid Public key: {0}")]
    InvalidPubkey(String),
    #[error("Missing metadata field: {0}")]
    MissingMetadatafield(&'static str),
    #[error("Invalid metadata field: {0}")]
    InvalidMetadata(String),
    #[error("Solana RPC error: {0}")]
    RpcError(String),
    #[error("Instruction error: {0}")]
    InstructionError(String),
    #[error("Invalid keypair: {0}")]
    InvalidKeypair(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

// use thiserror::Error;

// /// Custom error type for handling various failure cases
// #[derive(Debug, Error)]
// pub enum BubblegumError {
//     #[error("Invalid pubkey: {0}")]
//     InvalidPubkey(String),
//     #[error("RPC error: {0}")]
//     RpcError(String),
//     #[error("Serialization error: {0}")]
//     SerializationError(String),
//     #[error("Instruction error: {0}")]
//     InstructionError(String),
//     #[error("Invalid metadata: {0}")]
//     InvalidMetadata(String),
//     #[error("Invalid keypair: {0}")]
//     InvalidKeypair(String),
// }
