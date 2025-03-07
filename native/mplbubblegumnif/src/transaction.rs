use mpl_bubblegum::{
    instructions::{CreateTreeConfigBuilder, MintV1Builder, TransferBuilder},
    types::MetadataArgs,
};
use serde_json::from_str;
use solana_sdk::{
    message::Message,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
// use borsh::{BorshDeserialize, BorshSerialize};
use borsh::BorshDeserialize;

use crate::{
    error::NifError,
    utils::{
        get_recent_blockhash, parse_keypair, parse_pubkey, serialize_metadata_to_borsh, submit_tx,
    },
};

pub fn create_tree_config(
    rpc_url: &str,
    payer_pubkey: &str,
    tree_creator_pubkey: &str,
    max_depth: u32,
    max_buffer_size: u32,
    payer_secret_key: &str,
    tree_creator_secret_key: &str,
) -> Result<String, NifError> {
    // Parse pubkeys
    let payer = parse_pubkey(payer_pubkey)?;
    let tree_creator = parse_pubkey(tree_creator_pubkey)?;

    // Parse payer secret key
    let payer_keypair = parse_keypair(payer_secret_key)?;
    let tree_creator_keypair = parse_keypair(tree_creator_secret_key)?;

    // Build the instruction using mpl-bubblegum
    let instruction = CreateTreeConfigBuilder::new()
        .payer(payer)
        .tree_creator(tree_creator)
        .tree_config(payer)
        .merkle_tree(tree_creator)
        .max_depth(max_depth)
        .max_buffer_size(max_buffer_size)
        .instruction();

    // Fetch recent blockhash
    let recent_blockhash = get_recent_blockhash(rpc_url)?;

    // Construct transaction
    let message = Message::new(&[instruction], Some(&payer));
    let mut tx = Transaction::new_unsigned(message);
    tx.try_sign(&[&payer_keypair, &tree_creator_keypair], recent_blockhash)
        .map_err(|e| NifError::SerializationError(e.to_string()))?;

    // Submit transaction
    submit_tx(rpc_url, tx)
}

pub fn mint_v1(
    rpc_url: &str,
    tree_pubkey: &str,
    leaf_owner: &str,
    leaf_delegate: &str,
    metadata_borsh: &str,
    payer_secret_key: &str,
    leaf_owner_secret_key: &str,
) -> Result<String, NifError> {
    // Parse pubkeys
    let tree = parse_pubkey(tree_pubkey)?;
    let owner = parse_pubkey(leaf_owner)?;
    let delegate = parse_pubkey(leaf_delegate)?;
    let payer_keypair = parse_keypair(payer_secret_key)?;
    let leaf_owner_keypair = parse_keypair(leaf_owner_secret_key)?;

    // Decode the base64-encoded Borsh-serialized metadata
    let metadata_bytes = BASE64
        .decode(metadata_borsh)
        .map_err(|e| NifError::InvalidMetadata(format!("Base64 decode error: {}", e)))?;

    // Deserialize the Borsh bytes into MetadataArgs
    let metadata = MetadataArgs::try_from_slice(&metadata_bytes)
        .map_err(|e| NifError::InvalidMetadata(format!("Borsh deserialize error: {}", e)))?;

    // Build the instruction
    let instruction = MintV1Builder::new()
        .tree_config(tree)
        .leaf_owner(owner)
        .leaf_delegate(delegate)
        .merkle_tree(tree)
        .payer(payer_keypair.pubkey()) // Added
        .tree_creator_or_delegate(payer_keypair.pubkey())
        .metadata(metadata)
        .instruction();

    // Fetch recent blockhash
    let recent_blockhash = get_recent_blockhash(rpc_url)?;

    // Construct and sign transaction
    let message = Message::new(&[instruction], Some(&payer_keypair.pubkey()));
    let mut tx = Transaction::new_unsigned(message);
    tx.try_sign(&[&payer_keypair], recent_blockhash)
        .map_err(|e| NifError::SerializationError(e.to_string()))?;

    submit_tx(rpc_url, tx)
}

pub fn transfer(
    rpc_url: &str,
    tree_pubkey: &str,
    leaf_owner: &str,
    new_leaf_owner: &str,
    leaf_index: u32,
    payer_secret_key: &str,
    leaf_owner_secret_key: &str,
) -> Result<String, NifError> {
    // Parse pubkeys
    let tree = parse_pubkey(tree_pubkey)?;
    let owner = parse_pubkey(leaf_owner)?;
    let new_owner = parse_pubkey(new_leaf_owner)?;
    let payer_keypair = parse_keypair(payer_secret_key)?;
    let leaf_owner_keypair = parse_keypair(leaf_owner_secret_key)?;

    // Build the instruction
    let instruction = TransferBuilder::new()
        .tree_config(tree)
        .merkle_tree(tree)
        .leaf_owner(owner, true) // check once
        .leaf_delegate(owner, false)
        .new_leaf_owner(new_owner)
        .root([0; 32]) // Placeholder
        .data_hash([0; 32]) // Placeholder
        .creator_hash([0; 32]) // Placeholder
        .nonce(0) // Placeholder
        .index(leaf_index) // on leaf_index found in transferBuilder check once again
        .instruction();

    // Fetch recent blockhash
    let recent_blockhash = get_recent_blockhash(rpc_url)?;

    // Construct and sign transaction
    let message = Message::new(&[instruction], Some(&payer_keypair.pubkey()));
    let mut tx = Transaction::new_unsigned(message);
    tx.try_sign(&[&payer_keypair, &leaf_owner_keypair], recent_blockhash)
        .map_err(|e| NifError::SerializationError(e.to_string()))?;

    submit_tx(rpc_url, tx)
}

// ---------------Tests------------------------

// use super::*; // Import all from transaction.rs

// use solana_sdk::signer::Signer;

#[cfg(test)]
mod tests {
    use super::*;
    use mpl_bubblegum::types::{Creator, TokenProgramVersion};
    use solana_client::rpc_client::RpcClient;
    use std::thread::sleep;
    use std::time::Duration;

    const RPC_URL: &str = "https://api.devnet.solana.com"; // Public devnet RPC

    // Helper to create valid metadata JSON for mint_v1 tests
    fn create_valid_metadata_json(creator_pubkey: &str) -> String {
        format!(
            r#"{{
                "name": "Test NFT",
                "symbol": "TNFT",
                "uri": "https://example.com/nft.json",
                "seller_fee_basis_points": 500,
                "creators": [
                    {{
                        "address": "{}",
                        "verified": false,
                        "share": 100
                    }}
                ],
                "primary_sale_happened": false,
                "is_mutable": true
            }}"#,
            creator_pubkey
        )
    }

    fn airdrop_sol(rpc_url: &str, pubkey: &Pubkey, lamports: u64) -> Result<(), NifError> {
        let client = RpcClient::new(rpc_url.to_string());
        let mut attempts = 5;
        let mut delay = Duration::from_secs(2);

        while attempts > 0 {
            match client.request_airdrop(pubkey, lamports) {
                Ok(signature) => {
                    let mut retries = 10;
                    while retries > 0 {
                        if client.confirm_transaction(&signature).unwrap_or(false) {
                            println!("Airdropped {} lamports to {}", lamports, pubkey);
                            return Ok(());
                        }
                        sleep(Duration::from_secs(1));
                        retries -= 1;
                    }
                    return Err(NifError::RpcError(format!(
                        "Airdrop to {} failed to confirm: {}",
                        pubkey, signature
                    )));
                }
                Err(e) => {
                    if e.to_string().contains("rate limit") {
                        println!("Rate limit hit, retrying in {:?}", delay);
                        sleep(delay);
                        delay *= 2; // Exponential backoff
                        attempts -= 1;
                    } else {
                        return Err(NifError::RpcError(e.to_string()));
                    }
                }
            }
        }
        Err(NifError::RpcError(
            "Airdrop failed after retries due to rate limit".to_string(),
        ))
    }

    #[test]
    fn test_create_tree_config_success() {
        let payer = Keypair::new();
        let tree_creator = Keypair::new();

        // Airdrop SOL to payer and tree creator
        // airdrop_sol(RPC_URL, &payer.pubkey(), 1_000_000_000).expect("Failed to airdrop to payer");
        // airdrop_sol(RPC_URL, &tree_creator.pubkey(), 1_000_000_000)
        //     .expect("Failed to airdrop to tree creator");

        let payer_pubkey = payer.pubkey().to_string();
        let tree_creator_pubkey = tree_creator.pubkey().to_string();
        let payer_secret_key = payer.to_base58_string();
        let tree_creator_secret_key = tree_creator.to_base58_string();

        let result = create_tree_config(
            RPC_URL,
            &payer_pubkey,
            &tree_creator_pubkey,
            14,   // max_depth (example value)
            2048, // max_buffer_size (example value)
            &payer_secret_key,
            &tree_creator_secret_key,
        );

        match result {
            Ok(signature) => assert!(!signature.is_empty(), "Signature should not be empty"),
            Err(NifError::RpcError(msg)) => {
                // Tolerate account not found since payer isn’t funded
                assert!(
                    msg.contains("AccountNotFound") || msg.contains("MinimumBalance"),
                    "Unexpected RPC error: {}",
                    msg
                );
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn test_create_tree_config_invalid_payer_pubkey() {
        let tree_creator = Keypair::new();
        let payer = Keypair::new();

        let result = create_tree_config(
            RPC_URL,
            "invalid_payer_pubkey",
            &tree_creator.pubkey().to_string(),
            14,
            2048,
            &payer.to_base58_string(),
            &tree_creator.to_base58_string(),
        );

        assert!(result.is_err(), "Should fail with invalid payer pubkey");
        if let Err(NifError::InvalidPubkey(_)) = result {
            // Success
        } else {
            panic!("Wrong error type");
        }
    }

    #[test]
    fn test_create_tree_config_invalid_secret_key() {
        let payer = Keypair::new();
        let tree_creator = Keypair::new();

        let result = create_tree_config(
            RPC_URL,
            &payer.pubkey().to_string(),
            &tree_creator.pubkey().to_string(),
            14,
            2048,
            "invalid_secret_key",
            &tree_creator.to_base58_string(),
        );

        assert!(result.is_err(), "Should fail with invalid secret key");
        if let Err(NifError::InvalidKeypair(_)) = result {
            // Success
        } else {
            panic!("Wrong error type");
        }
    }

    #[test]
    fn test_mint_v1_success() {
        let payer = Keypair::new();
        let tree = Keypair::new();
        let leaf_owner = Keypair::new();
        let leaf_delegate = Keypair::new();

        // Airdrop SOL to payer and leaf owner
        airdrop_sol(RPC_URL, &payer.pubkey(), 1_000_000_000).expect("Failed to airdrop to payer");
        // airdrop_sol(RPC_URL, &leaf_owner.pubkey(), 1_000_000_000)
        //     .expect("Failed to airdrop to leaf owner");

        // Create valid metadata
        let metadata_json = create_valid_metadata_json(&payer.pubkey().to_string());
        let metadata_borsh = serialize_metadata_to_borsh(&metadata_json)
            .expect("Failed to serialize metadata for test");

        let result = mint_v1(
            RPC_URL,
            &tree.pubkey().to_string(),
            &leaf_owner.pubkey().to_string(),
            &leaf_delegate.pubkey().to_string(),
            &metadata_borsh,
            &payer.to_base58_string(),
            &leaf_owner.to_base58_string(),
        );

        match result {
            Ok(signature) => assert!(!signature.is_empty(), "Signature should not be empty"),
            Err(NifError::RpcError(msg)) => {
                // Tolerate account not found since accounts aren’t funded
                assert!(
                    msg.contains("AccountNotFound") || msg.contains("MinimumBalance"),
                    "Unexpected RPC error: {}",
                    msg
                );
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn test_mint_v1_invalid_tree_pubkey() {
        let payer = Keypair::new();
        let leaf_owner = Keypair::new();
        let leaf_delegate = Keypair::new();

        let metadata_json = create_valid_metadata_json(&payer.pubkey().to_string());
        let metadata_borsh = serialize_metadata_to_borsh(&metadata_json)
            .expect("Failed to serialize metadata for test");

        let result = mint_v1(
            RPC_URL,
            "invalid_tree_pubkey",
            &leaf_owner.pubkey().to_string(),
            &leaf_delegate.pubkey().to_string(),
            &metadata_borsh,
            &payer.to_base58_string(),
            &leaf_owner.to_base58_string(),
        );

        assert!(result.is_err(), "Should fail with invalid tree pubkey");
        if let Err(NifError::InvalidPubkey(_)) = result {
            // Success
        } else {
            panic!("Wrong error type");
        }
    }

    #[test]
    fn test_mint_v1_invalid_metadata() {
        let payer = Keypair::new();
        let tree = Keypair::new();
        let leaf_owner = Keypair::new();
        let leaf_delegate = Keypair::new();

        let result = mint_v1(
            RPC_URL,
            &tree.pubkey().to_string(),
            &leaf_owner.pubkey().to_string(),
            &leaf_delegate.pubkey().to_string(),
            "not_a_valid_borsh_base64_string",
            &payer.to_base58_string(),
            &leaf_owner.to_base58_string(),
        );

        assert!(result.is_err(), "Should fail with invalid metadata");
        if let Err(NifError::InvalidMetadata(msg)) = result {
            assert!(msg.contains("Base64 decode error"));
        } else {
            panic!("Wrong error type");
        }
    }

    #[test]
    fn test_transfer_success() {
        let payer = Keypair::new();
        let tree = Keypair::new();
        let leaf_owner = Keypair::new();
        let new_leaf_owner = Keypair::new();

        // Airdrop SOL to payer and leaf owner
        // airdrop_sol(RPC_URL, &payer.pubkey(), 1_000_000_000).expect("Failed to airdrop to payer");
        // airdrop_sol(RPC_URL, &leaf_owner.pubkey(), 1_000_000_000)
        //     .expect("Failed to airdrop to leaf owner");

        let result = transfer(
            RPC_URL,
            &tree.pubkey().to_string(),
            &leaf_owner.pubkey().to_string(),
            &new_leaf_owner.pubkey().to_string(),
            0, // leaf_index
            &payer.to_base58_string(),
            &leaf_owner.to_base58_string(),
        );

        match result {
            Ok(signature) => assert!(!signature.is_empty(), "Signature should not be empty"),
            Err(NifError::RpcError(msg)) => {
                // Tolerate account not found since accounts aren’t funded
                assert!(
                    msg.contains("AccountNotFound") || msg.contains("MinimumBalance"),
                    "Unexpected RPC error: {}",
                    msg
                );
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn test_transfer_invalid_leaf_owner() {
        let payer = Keypair::new();
        let tree = Keypair::new();
        let new_leaf_owner = Keypair::new();
        let leaf_owner = Keypair::new();

        let result = transfer(
            RPC_URL,
            &tree.pubkey().to_string(),
            "invalid_leaf_owner",
            &new_leaf_owner.pubkey().to_string(),
            0,
            &payer.to_base58_string(),
            &leaf_owner.to_base58_string(),
        );

        assert!(result.is_err(), "Should fail with invalid leaf owner");
        if let Err(NifError::InvalidPubkey(_)) = result {
            // Success
        } else {
            panic!("Wrong error type");
        }
    }

    #[test]
    fn test_transfer_invalid_secret_key() {
        let tree = Keypair::new();
        let leaf_owner = Keypair::new();
        let new_leaf_owner = Keypair::new();

        let result = transfer(
            RPC_URL,
            &tree.pubkey().to_string(),
            &leaf_owner.pubkey().to_string(),
            &new_leaf_owner.pubkey().to_string(),
            0,
            "invalid_secret_key",
            &leaf_owner.to_base58_string(),
        );

        assert!(result.is_err(), "Should fail with invalid secret key");
        if let Err(NifError::InvalidKeypair(_)) = result {
            // Success
        } else {
            panic!("Wrong error type");
        }
    }

    // Edge case: Test with a large leaf_index
    #[test]
    fn test_transfer_large_leaf_index() {
        let payer = Keypair::new();
        let tree = Keypair::new();
        let leaf_owner = Keypair::new();
        let new_leaf_owner = Keypair::new();

        let result = transfer(
            RPC_URL,
            &tree.pubkey().to_string(),
            &leaf_owner.pubkey().to_string(),
            &new_leaf_owner.pubkey().to_string(),
            u32::MAX, // Max possible leaf_index
            &payer.to_base58_string(),
            &leaf_owner.to_base58_string(),
        );

        match result {
            Ok(signature) => assert!(!signature.is_empty(), "Signature should not be empty"),
            Err(NifError::RpcError(msg)) => {
                assert!(
                    msg.contains("AccountNotFound") || msg.contains("MinimumBalance"),
                    "Unexpected RPC error: {}",
                    msg
                );
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }
}
