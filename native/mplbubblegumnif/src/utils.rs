use solana_client::rpc_client::RpcClient;
use solana_sdk::{hash::Hash, pubkey::Pubkey, signature::Keypair, transaction::Transaction};
use std::panic;
use std::str::FromStr;

use crate::error::NifError;

/// Helper to fetch recent blockhash from Solana devnet
pub fn get_recent_blockhash(rpc_url: &str) -> Result<Hash, NifError> {
    let client = RpcClient::new(rpc_url.to_string());
    client
        .get_latest_blockhash()
        .map_err(|e| NifError::RpcError(e.to_string()))
}

/// Helper to submit a transaction to Solana devnet
pub fn submit_tx(rpc_url: &str, tx: Transaction) -> Result<String, NifError> {
    let client = RpcClient::new(rpc_url.to_string());
    let signature = client
        .send_and_confirm_transaction(&tx)
        .map_err(|e| NifError::RpcError(e.to_string()))?;
    Ok(signature.to_string())
}

// fn get_recent_blockhash(rpc_url: &str) -> Result<solana_sdk::hash::Hash, NifError> {
//     let client = RpcClient::new(rpc_url.to_string());
//     client
//         .get_latest_blockhash()
//         .map_err(|e| NifError::RpcError(e.to_string()))
// }

// /// Helper to submit a serialized transaction to Solana devnet/mainnet
// fn submit_tx(rpc_url: &str, serialized_tx: Vec<u8>) -> Result<String, NifError> {
//     let client = RpcClient::new(rpc_url.to_string());
//     let tx: Transaction = bincode::deserialize(&serialized_tx)
//         .map_err(|e| NifError::SerializationError(e.to_string()))?;
//     let signature = client
//         .send_and_confirm_transaction(&tx)
//         .map_err(|e| NifError::RpcError(e.to_string()))?;
//     Ok(signature.to_string())
// }

/// Helper to parse a base58-encoded secret key into a Keypair
pub fn parse_keypair(secret_key: &str) -> Result<Keypair, NifError> {
    // Keypair::from_base58_string(secret_key).map_err(|e| NifError::InvalidKeypair(e.to_string()))

    // Use `catch_unwind` to handle potential panics
    let result = panic::catch_unwind(|| Keypair::from_base58_string(secret_key));

    match result {
        Ok(keypair) => Ok(keypair),
        Err(_) => Err(NifError::InvalidKeypair("Invalid secret key".to_string())),
    }
}

/// Helper to parse a base58-encoded public key into a Pubkey
pub fn parse_pubkey(pubkey: &str) -> Result<Pubkey, NifError> {
    Pubkey::from_str(pubkey).map_err(|e| NifError::InvalidPubkey(e.to_string()))
}
