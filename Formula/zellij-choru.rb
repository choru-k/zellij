class ZellijChoru < Formula
  desc "Zellij fork with agent-focused streaming, stacked-header, and pane-style improvements"
  homepage "https://github.com/choru-k/zellij"
  url "https://github.com/choru-k/zellij/archive/578fb91a096c37fb6dc249d446cda44c12c67f47.tar.gz"
  version "0.44.0-choru.1"
  sha256 "18d811e986385fbb34a71c10316abdbba84c3c8b64e81f75b1c03b5add20c928"
  license "MIT"
  head "https://github.com/choru-k/zellij.git", branch: "main"

  depends_on "rust" => :build
  depends_on "openssl@3"

  on_linux do
    depends_on "zlib-ng-compat"
  end

  conflicts_with "zellij", because: "both install a zellij executable"

  def install
    ENV["OPENSSL_DIR"] = Formula["openssl@3"].opt_prefix
    ENV["OPENSSL_NO_VENDOR"] = "1"

    system "cargo", "install", *std_cargo_args

    generate_completions_from_executable(bin/"zellij", "setup", "--generate-completion")
  end

  test do
    assert_match "pane_synchronized_output_ignore_commands",
                 shell_output("#{bin}/zellij setup --dump-config")
    assert_match "zellij 0.44.0", shell_output("#{bin}/zellij --version")
  end
end
