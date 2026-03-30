class ZellijChoru < Formula
  desc "Zellij fork with streaming and pane UX improvements"
  homepage "https://github.com/choru-k/zellij"
  url "https://github.com/choru-k/zellij/archive/refs/tags/v0.44.0-choru.1.tar.gz"
  version "0.44.0-choru.1"
  sha256 "1b47c874178392c7cdbccbf3d9e8b669c5256f6c45a47945dea2900a53d87880"
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
    assert_match "zellij #{version}", shell_output("#{bin}/zellij --version")
  end
end
