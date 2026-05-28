class Cirbinius < Formula
  desc "Zero-knowledge proof circuit builder and prover"
  homepage "https://github.com/elonmasai7/CirBinius"
  url "https://github.com/elonmasai7/CirBinius/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "0000000000000000000000000000000000000000000000000000000000000000"
  license "Apache-2.0"
  head "https://github.com/elonmasai7/CirBinius.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: "crates/cirbinius-api")
    system "cargo", "install", *std_cargo_args(path: "crates/cirbinius-cli")
    bin.install "cirbinius-api"
    bin.install "cirbinius"
  end

  test do
    assert_match "cirbinius-api", shell_output("#{bin}/cirbinius-api --version 2>&1", 1)
  end
end
