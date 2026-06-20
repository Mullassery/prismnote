# Homebrew formula for PrismNote
# For use in custom tap: homebrew-prismnote

class Prismnote < Formula
  desc "Enterprise-grade, open-source data science notebook platform"
  homepage "https://github.com/Mullassery/prismnote"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/Mullassery/prismnote/releases/download/v1.0.0/prismnote-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_ARM64_SHA256"
    end
    on_intel do
      url "https://github.com/Mullassery/prismnote/releases/download/v1.0.0/prismnote-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_X86_64_SHA256"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/Mullassery/prismnote/releases/download/v1.0.0/prismnote-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_ARM64_LINUX_SHA256"
    end
    on_intel do
      url "https://github.com/Mullassery/prismnote/releases/download/v1.0.0/prismnote-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_X86_64_LINUX_SHA256"
    end
  end

  depends_on "python@3.11" => :recommended

  def install
    bin.install "prismnote"

    # Create completion scripts
    bash_completion.install "completions/prismnote.bash" => "prismnote"
    zsh_completion.install "completions/_prismnote" => "_prismnote"
    fish_completion.install "completions/prismnote.fish"
  end

  def post_install
    puts "PrismNote installed successfully!"
    puts ""
    puts "Quick start:"
    puts "  prismnote                    # Start server on http://localhost:8000"
    puts "  prismnote --port 3000        # Use custom port"
    puts "  prismnote --data /custom/dir # Use custom data directory"
    puts ""
    puts "Documentation: https://github.com/Mullassery/prismnote/wiki"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/prismnote --version")
  end
end
