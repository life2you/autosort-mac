class AutosortMac < Formula
  desc "Automatically organize Desktop and Downloads files on macOS"
  homepage "https://github.com/life2you/autosort-mac"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/life2you/autosort-mac/releases/download/v0.1.0/autosort-mac-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_ARM64_SHA256"
    end

    on_intel do
      url "https://github.com/life2you/autosort-mac/releases/download/v0.1.0/autosort-mac-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_X64_SHA256"
    end
  end

  def install
    bin.install "autosort-mac"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/autosort-mac --version")
  end
end
