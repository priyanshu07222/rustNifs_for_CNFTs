# Rust NIFs for Elixir: Metaplex Bubblegum Integration

This project provides Native Implemented Functions (NIFs) to bridge the `mpl-bubblegum` Rust crate from Metaplex to Elixir, enabling Elixir developers to create, mint, and transfer compressed NFTs on Solana.

## Features
- Create Merkle tree configurations for compressed NFTs.
- Mint compressed NFTs using `MintV1`.
- Transfer compressed NFTs between owners.
- Error handling and transaction submission to Solana devnet.

## Setup Instructions

### Prerequisites
- Elixir 1.12+ and Erlang/OTP 24+
- Rust 1.65+ with `cargo`
- Solana devnet keypairs for testing

### Dependencies
Add to your `mix.exs`:
```elixir
defp deps do
  [{:jason, "~> 1.4"}]
end