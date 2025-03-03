defmodule RustNifsForElixirTest do
  use ExUnit.Case
  doctest RustNifsForElixir

  test "greets the world" do
    assert RustNifsForElixir.hello() == :world
  end
end
