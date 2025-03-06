// use mpl_bubblegum::{
//     instructions::{create_tree_config, mint_v1, transfer},
//     state::MetadataArgs,
// };
use mpl_bubblegum::{
    instructions::{CreateTreeConfigBuilder, MintV1Builder, TransferBuilder},
    types::MetadataArgs,
};
// use rustler::{NifResult, NifStruct, Term};
// use solana_program::
use serde_json::from_str;
use solana_sdk::{
    message::Message,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

use crate::{
    error::NifError,
    utils::{get_recent_blockhash, parse_keypair, parse_pubkey, submit_tx},
};

// #[rustler::nif]
// fn create_tree_config(
//     env: Env,
//     rpc_url: String,
//     payer_pubkey: String,
//     tree_creator_pubkey: String,
//     max_depth: u32,
//     max_buffer_size: u32,
//     payer_secret_key: String,
// ) -> Term {
//     match create_tree_config_internal(
//         &rpc_url,
//         &payer_pubkey,
//         &tree_creator_pubkey,
//         max_depth,
//         max_buffer_size,
//         &payer_secret_key,
//     ) {
//         Ok(signature) => (atoms::ok(), signature).encode(env),
//         Err(e) => (atoms::error(), e.to_string()).encode(env),
//     }
// }

pub fn create_tree_config(
    rpc_url: &str,
    payer_pubkey: &str,
    tree_creator_pubkey: &str,
    max_depth: u32,
    max_buffer_size: u32,
    payer_secret_key: &str,
) -> Result<String, NifError> {
    // Parse pubkeys
    let payer = parse_pubkey(payer_pubkey)?;
    let tree_creator = parse_pubkey(tree_creator_pubkey)?;

    // Parse payer secret key
    let payer_keypair = parse_keypair(payer_secret_key)?;

    // Build the instruction using mpl-bubblegum
    let instruction = CreateTreeConfigBuilder::new()
        .payer(payer)
        .tree_creator(tree_creator)
        .max_depth(max_depth)
        .max_buffer_size(max_buffer_size)
        .instruction();

    // Fetch recent blockhash
    let recent_blockhash = get_recent_blockhash(rpc_url)?;

    // Construct transaction
    let message = Message::new(&[instruction], Some(&payer));
    let mut tx = Transaction::new_unsigned(message);
    tx.try_sign(&[&payer_keypair], recent_blockhash)
        .map_err(|e| NifError::SerializationError(e.to_string()))?;

    // Submit transaction
    submit_tx(rpc_url, tx)
}

// /// NIF: Mints a compressed NFT into a Merkle tree
// #[rustler::nif]
// fn mint_v1(
//     env: Env,
//     rpc_url: String,
//     tree_pubkey: String,
//     leaf_owner: String,
//     leaf_delegate: String,
//     metadata_json: String,
//     payer_secret_key: String,
// ) -> Term {
//     match mint_v1_internal(
//         &rpc_url,
//         &tree_pubkey,
//         &leaf_owner,
//         &leaf_delegate,
//         &metadata_json,
//         &payer_secret_key,
//     ) {
//         Ok(signature) => (atoms::ok(), signature).encode(env),
//         Err(e) => (atoms::error(), e.to_string()).encode(env),
//     }
// }

pub fn mint_v1(
    rpc_url: &str,
    tree_pubkey: &str,
    leaf_owner: &str,
    leaf_delegate: &str,
    metadata_json: &str,
    payer_secret_key: &str,
) -> Result<String, NifError> {
    // Parse pubkeys
    let tree = parse_pubkey(tree_pubkey)?;
    let owner = parse_pubkey(leaf_owner)?;
    let delegate = parse_pubkey(leaf_delegate)?;
    let payer_keypair = parse_keypair(payer_secret_key)?;

    // Parse metadata JSON into MetadataArgs
    let metadata: MetadataArgs =
        from_str(metadata_json).map_err(|e| NifError::InvalidMetadata(e.to_string()))?;

    // Build the instruction
    let instruction = MintV1Builder::new()
        .tree_config(tree)
        .leaf_owner(owner)
        .leaf_delegate(delegate)
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

// / NIF: Transfers a compressed NFT to a new owner
// #[rustler::nif]
// fn transfer(
//     env: Env,
//     rpc_url: String,
//     tree_pubkey: String,
//     leaf_owner: String,
//     new_leaf_owner: String,
//     leaf_index: u64,
//     payer_secret_key: String,
// ) -> Term {
//     match transfer_internal(
//         &rpc_url,
//         &tree_pubkey,
//         &leaf_owner,
//         &new_leaf_owner,
//         leaf_index,
//         &payer_secret_key,
//     ) {
//         Ok(signature) => (atoms::ok(), signature).encode(env),
//         Err(e) => (atoms::error(), e.to_string()).encode(env),
//     }
// }

pub fn transfer(
    rpc_url: &str,
    tree_pubkey: &str,
    leaf_owner: &str,
    new_leaf_owner: &str,
    leaf_index: u32,
    payer_secret_key: &str,
) -> Result<String, NifError> {
    // Parse pubkeys
    let tree = parse_pubkey(tree_pubkey)?;
    let owner = parse_pubkey(leaf_owner)?;
    let new_owner = parse_pubkey(new_leaf_owner)?;
    let payer_keypair = parse_keypair(payer_secret_key)?;

    // Build the instruction
    let instruction = TransferBuilder::new()
        .tree_config(tree)
        .leaf_owner(owner, true) // check once
        .new_leaf_owner(new_owner)
        .index(leaf_index) // on leaf_index found in transferBuilder check once again
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
