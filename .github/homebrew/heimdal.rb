# Homebrew Formula for Heimdal
# This file should be placed in a homebrew tap repository
# For example: https://github.com/limistah/homebrew-tap

class Heimdal < Formula
  desc "Universal dotfile and system configuration manager"
  homepage "https://github.com/limistah/heimdal"
  url "https://github.com/limistah/heimdal/archive/refs/tags/v1.0.0.tar.gz"
  sha256 "REPLACE_WITH_ACTUAL_SHA256"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "heimdal", shell_output("#{bin}/heimdal --version")
  end
end

# Installation instructions:
# 1. Create a tap repository: https://github.com/limistah/homebrew-tap
# 2. Add this formula to the repository as Formula/heimdal.rb
# 3. Users can install with: brew tap limistah/tap && brew install heimdal
#
# To update the formula after a new release:
# 1. Update the version number in the url
# 2. Download the tarball and calculate SHA256:
#    curl -L https://github.com/limistah/heimdal/archive/refs/tags/v1.0.0.tar.gz | shasum -a 256
# 3. Update the sha256 value
# 4. Commit and push to the tap repository
