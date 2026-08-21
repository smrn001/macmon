class Macmon < Formula
  desc "Ultra-lightweight system monitor for macOS"
  homepage "https://github.com/smrn001/macmon"
  url "https://github.com/smrn001/macmon/archive/refs/tags/v0.1.0.tar.gz"
  version "0.1.0"
  license "MIT"

  # NOTE: fill with `shasum -a 256 <tarball>` after publishing v0.1.0.
  sha256 "0000000000000000000000000000000000000000000000000000000000000000"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  def caveats
    <<~EOS
      macmon is a terminal application. Run it with:

        macmon

      Prebuilt binaries are also available on GitHub Releases:
        https://github.com/smrn001/macmon/releases
    EOS
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/macmon --version")
  end
end
