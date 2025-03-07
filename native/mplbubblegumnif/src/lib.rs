mod error;
mod transaction;
mod utils;
use crate::{
    error::NifError,
    transaction::{create_tree_config, mint_v1, transfer},
    utils::serialize_metadata_to_borsh,
};
use rustler::{Encoder, Env, Term};

// Define atoms for Elixir interop
mod atoms {
    rustler::atoms! {
        ok,
        error
    }
}

// Register NIF functions
rustler::init!(
    "Elixir.BubblegumNIF",
    [
        create_tree_config_nif,
        mint_v1_nif,
        transfer_nif,
        serialize_metadata_to_borsh_nif
    ]
);

/// NIF: Serializes metadata JSON into Borsh format
#[rustler::nif]
fn serialize_metadata_to_borsh_nif(env: Env, metadata_json: String) -> Term {
    match serialize_metadata_to_borsh(&metadata_json) {
        Ok(borsh_data) => (atoms::ok(), borsh_data).encode(env),
        Err(e) => (atoms::error(), e.to_string()).encode(env),
    }
}

/// NIF: Creates a tree config for compressed NFTs and submits the transaction
#[rustler::nif]
fn create_tree_config_nif(
    env: Env,
    rpc_url: String,
    payer_pubkey: String,
    tree_creator_pubkey: String,
    max_depth: u32,
    max_buffer_size: u32,
    payer_secret_key: String,
    tree_creator_secret_key: String,
) -> Term {
    match create_tree_config(
        &rpc_url,
        &payer_pubkey,
        &tree_creator_pubkey,
        max_depth,
        max_buffer_size,
        &payer_secret_key,
        &tree_creator_secret_key,
    ) {
        Ok(signature) => (atoms::ok(), signature).encode(env),
        Err(e) => (atoms::error(), e.to_string()).encode(env),
    }
}

/// NIF: Mints a compressed NFT and submits the transaction
#[rustler::nif]
fn mint_v1_nif(
    env: Env,
    rpc_url: String,
    tree_pubkey: String,
    leaf_owner: String,
    leaf_delegate: String,
    metadata_borsh: String,
    payer_secret_key: String,
    leaf_owner_secret_key: String,
) -> Term {
    match mint_v1(
        &rpc_url,
        &tree_pubkey,
        &leaf_owner,
        &leaf_delegate,
        &metadata_borsh,
        &payer_secret_key,
        &leaf_owner_secret_key,
    ) {
        Ok(signature) => (atoms::ok(), signature).encode(env),
        Err(e) => (atoms::error(), e.to_string()).encode(env),
    }
}

/// NIF: Transfers a compressed NFT and submits the transaction
#[rustler::nif]
fn transfer_nif(
    env: Env,
    rpc_url: String,
    tree_pubkey: String,
    leaf_owner: String,
    new_leaf_owner: String,
    leaf_index: u32,
    payer_secret_key: String,
    leaf_owner_secret_key: String,
) -> Term {
    match transfer(
        &rpc_url,
        &tree_pubkey,
        &leaf_owner,
        &new_leaf_owner,
        leaf_index,
        &payer_secret_key,
        &leaf_owner_secret_key,
    ) {
        Ok(signature) => (atoms::ok(), signature).encode(env),
        Err(e) => (atoms::error(), e.to_string()).encode(env),
    }
}

// rustler::init!("Elixir.MplBubblegumNif", [add]);
