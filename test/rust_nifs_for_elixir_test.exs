defmodule RustNifsForElixirTest do
  use ExUnit.Case, async: true

  @rpc_url "https://devnet.helius-rpc.com/?api-key=b55951f7-cd70-411d-8962-abbd2e2c7877"

  # Replace these with real devnet keypairs for testing
  @payer_secret_key "your-base58-secret-key"
  @payer_pubkey "your-base58-pubkey"
  @tree_creator_pubkey "another-base58-pubkey"
  @leaf_owner "leaf-owner-pubkey"
  @leaf_delegate "leaf-delegate-pubkey"
  @new_leaf_owner "new-leaf-owner-pubkey"

  test "create_tree_config constructs and submits a transaction" do
    assert {:ok, signature} = RustNifsForElixir.create_tree_config(
      @rpc_url,
      @payer_pubkey,
      @tree_creator_pubkey,
      14,
      2048,
      @payer_secret_key
    )
    assert is_binary(signature)
  end

  test "serialize_metadata_to_borsh converts JSON to Borsh" do
    metadata_json = Jason.encode!(%{
      "name" => "Test NFT",
      "symbol" => "TNFT",
      "uri" => "https://example.com/test.json",
      "seller_fee_basis_points" => 500,
      "creators" => [%{"address" => @payer_pubkey, "verified" => false, "share" => 100}],
      "primary_sale_happened" => false,
      "is_mutable" => true
    })

    assert {:ok, borsh_data} = RustNifsForElixir.serialize_metadata_to_borsh(metadata_json)
    assert is_binary(borsh_data)
  end

  test "mint_v1 mints a compressed NFT" do
    metadata_json = Jason.encode!(%{
      "name" => "Test NFT",
      "symbol" => "TNFT",
      "uri" => "https://example.com/test.json",
      "seller_fee_basis_points" => 500,
      "creators" => [%{"address" => @payer_pubkey, "verified" => false, "share" => 100}],
      "primary_sale_happened" => false,
      "is_mutable" => true
    })

    {:ok, metadata_borsh} = RustNifsForElixir.serialize_metadata_to_borsh(metadata_json)
    {:ok, tree_signature} = RustNifsForElixir.create_tree_config(
      @rpc_url,
      @payer_pubkey,
      @tree_creator_pubkey,
      14,
      2048,
      @payer_secret_key
    )

    # Assume tree_pubkey is derived from tree_signature or known
    tree_pubkey = "some-tree-pubkey" # Replace with actual tree pubkey

    assert {:ok, mint_signature} = RustNifsForElixir.mint_v1(
      @rpc_url,
      tree_pubkey,
      @leaf_owner,
      @leaf_delegate,
      metadata_borsh,
      @payer_secret_key
    )
    assert is_binary(mint_signature)
  end

  test "transfer transfers a compressed NFT" do
    # Assume a minted NFT exists at leaf_index 0
    assert {:ok, transfer_signature} = RustNifsForElixir.transfer(
      @rpc_url,
      "some-tree-pubkey", # Replace with actual tree pubkey
      @leaf_owner,
      @new_leaf_owner,
      0,
      @payer_secret_key
    )
    assert is_binary(transfer_signature)
  end
end
