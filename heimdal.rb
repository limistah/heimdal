class Heimdal < Formula
  desc "Universal dotfile and system configuration manager"
  homepage "https://github.com/limistah/heimdal"
  version "1.0.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/limistah/heimdal/releases/download/v1.0.0/heimdal-darwin-arm64.tar.gz"
      sha256 "e553054d52c1115e72e941674f4d7546ca065b674450ea508a349740d9537c57"
    else
      url "https://github.com/limistah/heimdal/releases/download/v1.0.0/heimdal-darwin-amd64.tar.gz"
      sha256 "10d1c39a12e8a1c8b40bd9eff2f0ef34ed8015bf4d42437188ace1e7edddf396"
    end
  end

  def install
    bin.install "heimdal"
  end

  test do
    assert_match "heimdal 1.0.0", shell_output("#{bin}/heimdal --version")
  end
end
