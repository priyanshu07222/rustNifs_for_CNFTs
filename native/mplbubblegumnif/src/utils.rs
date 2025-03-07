use solana_client::rpc_client::RpcClient;
use solana_sdk::{hash::Hash, pubkey::Pubkey, signature::Keypair, transaction::Transaction};
use std::panic;
use std::str::FromStr;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use borsh::{BorshDeserialize, BorshSerialize};
use mpl_bubblegum::types::{Creator, MetadataArgs};

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

/// Helper to parse a base58-encoded secret key into a Keypair
pub fn parse_keypair(secret_key: &str) -> Result<Keypair, NifError> {
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

/// Helper to serialize metadata into Borsh format
pub fn serialize_metadata_to_borsh(metadata_json: &str) -> Result<String, NifError> {
    // Define a temporary struct to deserialize JSON
    #[derive(serde::Deserialize)]
    struct MetadataInput {
        name: String,
        symbol: String,
        uri: String,
        seller_fee_basis_points: u16,
        creators: Option<Vec<CreatorInput>>,
        primary_sale_happened: bool,
        is_mutable: bool,
    }

    #[derive(serde::Deserialize)]
    struct CreatorInput {
        address: String,
        verified: bool,
        share: u8,
    }

    // Parse JSON into MetadataInput
    let metadata_input: MetadataInput = serde_json::from_str(metadata_json)
        .map_err(|e| NifError::InvalidMetadata(format!("JSON parse error: {}", e)))?;

    // Convert to MetadataArgs
    let creators = metadata_input
        .creators
        .unwrap_or_default()
        .into_iter()
        .map(|c| {
            let address =
                Pubkey::from_str(&c.address).map_err(|e| NifError::InvalidPubkey(e.to_string()))?;
            Ok(Creator {
                address,
                verified: c.verified,
                share: c.share,
            })
        })
        .collect::<Result<Vec<Creator>, NifError>>()?;

    let metadata = MetadataArgs {
        name: metadata_input.name,
        symbol: metadata_input.symbol,
        uri: metadata_input.uri,
        seller_fee_basis_points: metadata_input.seller_fee_basis_points,
        creators,
        primary_sale_happened: metadata_input.primary_sale_happened,
        is_mutable: metadata_input.is_mutable,
        edition_nonce: None,
        uses: None,
        collection: None,
        token_standard: None,
        token_program_version: mpl_bubblegum::types::TokenProgramVersion::Original,
    };

    // Serialize to Borsh
    let metadata_bytes = metadata
        .try_to_vec()
        .map_err(|e| NifError::SerializationError(format!("Borsh serialize error: {}", e)))?;

    // Encode as base64
    let metadata_base64 = BASE64.encode(&metadata_bytes);
    Ok(metadata_base64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::NifError;
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
    use solana_sdk::signature::Signer;

    // Test constants
    const RPC_URL: &str =
        "https://devnet.helius-rpc.com/?api-key=b55951f7-cd70-411d-8962-abbd2e2c7877";
    const VALID_PUBKEY: &str = "11111111111111111111111111111111"; // Example base58 key

    #[test]
    fn test_get_recent_blockhash() {
        let result = get_recent_blockhash(RPC_URL);
        assert!(
            result.is_ok(),
            "Failed to get recent blockhash: {:?}",
            result.err()
        );
        let blockhash = result.unwrap();
        assert_eq!(blockhash.to_string().len(), 44, "Invalid blockhash length");
    }

    #[test]
    fn test_parse_keypair_valid() {
        // Generate a new keypair
        let original_keypair = Keypair::new();
        let base58_keypair = original_keypair.to_base58_string();

        let result = parse_keypair(&base58_keypair);
        assert!(
            result.is_ok(),
            "Failed to parse valid keypair: {:?}",
            result.err()
        );
        let keypair = result.unwrap();
        assert_eq!(
            keypair.pubkey().to_string().len(),
            44,
            "Invalid pubkey length"
        );
    }

    #[test]
    fn test_parse_keypair_invalid() {
        let result = parse_keypair("invalid_key");
        assert!(result.is_err(), "Should fail with invalid keypair");
        if let Err(NifError::InvalidKeypair(msg)) = result {
            assert_eq!(msg, "Invalid secret key");
        } else {
            panic!("Wrong error type");
        }
    }

    #[test]
    fn test_parse_pubkey_valid() {
        let result = parse_pubkey(VALID_PUBKEY);
        assert!(
            result.is_ok(),
            "Failed to parse valid pubkey: {:?}",
            result.err()
        );
        let pubkey = result.unwrap();
        assert_eq!(pubkey.to_string(), VALID_PUBKEY);
    }

    #[test]
    fn test_parse_pubkey_invalid() {
        let result = parse_pubkey("invalid_pubkey");
        assert!(result.is_err(), "Should fail with invalid pubkey");
        if let Err(NifError::InvalidPubkey(_)) = result {
            // Success
        } else {
            panic!("Wrong error type");
        }
    }

    #[test]
    fn test_serialize_metadata_to_borsh_valid() {
        let metadata_json = r#"
        {
            "name": "Test NFT",
            "symbol": "TNFT",
            "uri": "https://example.com/nft.json",
            "seller_fee_basis_points": 500,
            "creators": [
                {
                    "address": "11111111111111111111111111111111",
                    "verified": false,
                    "share": 100
                }
            ],
            "primary_sale_happened": false,
            "is_mutable": true
        }
    "#;

        let result = serialize_metadata_to_borsh(metadata_json);
        assert!(
            result.is_ok(),
            "Failed to serialize metadata: {:?}",
            result.err()
        );
        let base64_str = result.unwrap();
        assert!(!base64_str.is_empty(), "Base64 string should not be empty");

        // Decode to verify it's valid base64
        let decoded = BASE64.decode(&base64_str);
        assert!(decoded.is_ok(), "Invalid base64 output");
    }

    #[test]
    fn test_serialize_metadata_to_borsh_invalid_json() {
        let invalid_json = "not a json string";
        let result = serialize_metadata_to_borsh(invalid_json);
        assert!(result.is_err(), "Should fail with invalid JSON");
        if let Err(NifError::InvalidMetadata(_)) = result {
            // Success
        } else {
            panic!("Wrong error type");
        }
    }

    #[test]
    fn test_serialize_metadata_with_invalid_creator() {
        let metadata_json = r#"
        {
            "name": "Test NFT",
            "symbol": "TNFT",
            "uri": "https://example.com/nft.json",
            "seller_fee_basis_points": 500,
            "creators": [
                {
                    "address": "invalid_pubkey",
                    "verified": false,
                    "share": 100
                }
            ],
            "primary_sale_happened": false,
            "is_mutable": true
        }
    "#;

        let result = serialize_metadata_to_borsh(metadata_json);
        assert!(result.is_err(), "Should fail with invalid creator pubkey");
        if let Err(NifError::InvalidPubkey(_)) = result {
            // Success
        } else {
            panic!("Wrong error type");
        }
    }
}
