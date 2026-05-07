class AutosortMac < Formula
  desc "Automatically organize Desktop and Downloads files on macOS"
  homepage "https://github.com/life2you/autosort-mac"
  url "https://github.com/life2you/autosort-mac/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "SHA256_PLACEHOLDER"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: ".")
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/autosort-mac --version")
  end
end
