class Statguard < Formula
  desc "High-performance data quality, validation, and drift monitoring engine"
  homepage "https://github.com/Mullassery/statguard"
  license "MIT"

  url "https://files.pythonhosted.org/packages/statguard-0.1.0-cp313-cp313-macosx_10_12_x86_64.whl"
  sha256 "UPDATE_WITH_ACTUAL_SHA256"

  depends_on "python@3.11"

  def install
    venv = virtualenv_create(libexec, "python3.11")
    venv.pip_install "statguard==0.1.0"
    bin.install_symlink libexec/"bin/statguard"
  end

  test do
    system bin/"statguard", "--version"
  end
end
