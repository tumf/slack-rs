class SlackRs < Formula
  desc "Slack CLI tool with OAuth, profiles, and API access"
  homepage "https://github.com/tumf/slack-rs"
  url "https://github.com/tumf/slack-rs/archive/refs/tags/v0.1.67.tar.gz"
  sha256 "79b161aac0e15d466996fbb56fe9b87a15b747e001ae5fcd745722f48ce08bd1"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    system "#{bin}/slack-rs", "--version"
  end
end
