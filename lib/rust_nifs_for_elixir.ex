defmodule RustNifsForElixir do
  @moduledoc """
  A module to interact with Rust NIFs for creating, minting, and transferring compressed NFTs
  on Solana using the `mpl_bubblegum` library.

  This module provides an Elixir interface to the Rust NIFs defined in the `BubblegumNIF` native
  module. It handles Solana transactions for tree configuration, NFT minting, and transfers.
  """

  # Load the NIFs dynamically from the Rust library
  # The library name 'mplbubblegumnif' matches the Rust project name
  @on_load :load_nifs

  def load_nifs do
    :erlang.load_nif("./priv/native/libmplbubblegumnif", 0)
  end

  @doc """
  Creates a tree configuration for compressed NFTs on Solana.

  ## Parameters
  - `rpc_url`: The Solana RPC URL (e.g., "https://api.devnet.solana.com").
  - `payer_pubkey`: The base58-encoded public key of the payer.
  - `tree_creator_pubkey`: The base58-encoded public key of the tree creator.
  - `max_depth`: The maximum depth of the Merkle tree (u32).
  - `max_buffer_size`: The maximum buffer size for the tree (u32).
  - `payer_secret_key`: The base58-encoded secret key of the payer.

  ## Returns
  - `{:ok, signature}`: The transaction signature on success.
  - `{:error, reason}`: An error message if the operation fails.

  ## Examples
      iex> RustNifsForElixir.create_tree_config(
      ...>   "https://api.devnet.solana.com",
      ...>   "payer-pubkey",
      ...>   "tree-creator-pubkey",
      ...>   14,
      ...>   2048,
      ...>   "payer-secret-key"
      ...> )
      {:ok, "some-transaction-signature"}
  """
  @spec create_tree_config(
          String.t(),
          String.t(),
          String.t(),
          non_neg_integer(),
          non_neg_integer(),
          String.t()
        ) ::
          {:ok, String.t()} | {:error, String.t()}
  def create_tree_config(
        rpc_url,
        payer_pubkey,
        tree_creator_pubkey,
        max_depth,
        max_buffer_size,
        payer_secret_key
      ) do
    case create_tree_config_nif(
           rpc_url,
           payer_pubkey,
           tree_creator_pubkey,
           max_depth,
           max_buffer_size,
           payer_secret_key
         ) do
      {:ok, signature} -> {:ok, signature}
      {:error, reason} -> {:error, reason}
    end
  end

  @doc """
  Serializes NFT metadata from JSON to Borsh format for use in minting.

  ## Parameters
  - `metadata_json`: A JSON string containing the metadata (e.g., name, symbol, uri, etc.).

  ## Returns
  - `{:ok, borsh_data}`: The base64-encoded Borsh-serialized metadata.
  - `{:error, reason}`: An error message if serialization fails.

  ## Examples
      iex> metadata = Jason.encode!(%{
      ...>   "name" => "My NFT",
      ...>   "symbol" => "MNFT",
      ...>   "uri" => "https://example.com/metadata.json",
      ...>   "seller_fee_basis_points" => 500,
      ...>   "creators" => [%{"address" => "some-pubkey", "verified" => false, "share" => 100}],
      ...>   "primary_sale_happened" => false,
      ...>   "is_mutable" => true
      ...> })
      iex> RustNifsForElixir.serialize_metadata_to_borsh(metadata)
      {:ok, "base64-encoded-borsh-data"}
  """
  @spec serialize_metadata_to_borsh(String.t()) :: {:ok, String.t()} | {:error, String.t()}
  def serialize_metadata_to_borsh(metadata_json) do
    case serialize_metadata_to_borsh_nif(metadata_json) do
      {:ok, borsh_data} -> {:ok, borsh_data}
      {:error, reason} -> {:error, reason}
    end
  end

  @doc """
  Mints a compressed NFT on Solana.

  ## Parameters
  - `rpc_url`: The Solana RPC URL (e.g., "https://api.devnet.solana.com").
  - `tree_pubkey`: The base58-encoded public key of the Merkle tree.
  - `leaf_owner`: The base58-encoded public key of the leaf owner.
  - `leaf_delegate`: The base58-encoded public key of the leaf delegate.
  - `metadata_borsh`: The base64-encoded Borsh-serialized metadata (from `serialize_metadata_to_borsh/1`).
  - `payer_secret_key`: The base58-encoded secret key of the payer.

  ## Returns
  - `{:ok, signature}`: The transaction signature on success.
  - `{:error, reason}`: An error message if the operation fails.

  ## Examples
      iex> {:ok, metadata_borsh} = RustNifsForElixir.serialize_metadata_to_borsh("{\"name\":\"My NFT\"...")
      iex> RustNifsForElixir.mint_v1(
      ...>   "https://api.devnet.solana.com",
      ...>   "tree-pubkey",
      ...>   "leaf-owner",
      ...>   "leaf-delegate",
      ...>   metadata_borsh,
      ...>   "payer-secret-key"
      ...> )
      {:ok, "some-transaction-signature"}
  """
  @spec mint_v1(String.t(), String.t(), String.t(), String.t(), String.t(), String.t()) ::
          {:ok, String.t()} | {:error, String.t()}
  def mint_v1(rpc_url, tree_pubkey, leaf_owner, leaf_delegate, metadata_borsh, payer_secret_key) do
    case mint_v1_nif(
           rpc_url,
           tree_pubkey,
           leaf_owner,
           leaf_delegate,
           metadata_borsh,
           payer_secret_key
         ) do
      {:ok, signature} -> {:ok, signature}
      {:error, reason} -> {:error, reason}
    end
  end

  @doc """
  Transfers a compressed NFT on Solana.

  ## Parameters
  - `rpc_url`: The Solana RPC URL (e.g., "https://api.devnet.solana.com").
  - `tree_pubkey`: The base58-encoded public key of the Merkle tree.
  - `leaf_owner`: The base58-encoded public key of the current leaf owner.
  - `new_leaf_owner`: The base58-encoded public key of the new leaf owner.
  - `leaf_index`: The index of the leaf in the Merkle tree (u32).
  - `payer_secret_key`: The base58-encoded secret key of the payer.

  ## Returns
  - `{:ok, signature}`: The transaction signature on success.
  - `{:error, reason}`: An error message if the operation fails.

  ## Examples
      iex> RustNifsForElixir.transfer(
      ...>   "https://api.devnet.solana.com",
      ...>   "tree-pubkey",
      ...>   "current-leaf-owner",
      ...>   "new-leaf-owner",
      ...>   0,
      ...>   "payer-secret-key"
      ...> )
      {:ok, "some-transaction-signature"}
  """
  @spec transfer(String.t(), String.t(), String.t(), String.t(), non_neg_integer(), String.t()) ::
          {:ok, String.t()} | {:error, String.t()}
  def transfer(rpc_url, tree_pubkey, leaf_owner, new_leaf_owner, leaf_index, payer_secret_key) do
    case transfer_nif(
           rpc_url,
           tree_pubkey,
           leaf_owner,
           new_leaf_owner,
           leaf_index,
           payer_secret_key
         ) do
      {:ok, signature} -> {:ok, signature}
      {:error, reason} -> {:error, reason}
    end
  end

  # Private NIF stubs - these will be replaced by the loaded NIFs
  # If the NIFs fail to load, these will raise an error
  defp create_tree_config_nif(
         _rpc_url,
         _payer_pubkey,
         _tree_creator_pubkey,
         _max_depth,
         _max_buffer_size,
         _payer_secret_key
       ) do
    raise "NIF create_tree_config_nif/6 not loaded"
  end

  defp serialize_metadata_to_borsh_nif(_metadata_json) do
    raise "NIF serialize_metadata_to_borsh_nif/1 not loaded"
  end

  defp mint_v1_nif(
         _rpc_url,
         _tree_pubkey,
         _leaf_owner,
         _leaf_delegate,
         _metadata_borsh,
         _payer_secret_key
       ) do
    raise "NIF mint_v1_nif/6 not loaded"
  end

  defp transfer_nif(
         _rpc_url,
         _tree_pubkey,
         _leaf_owner,
         _new_leaf_owner,
         _leaf_index,
         _payer_secret_key
       ) do
    raise "NIF transfer_nif/6 not loaded"
  end
end
