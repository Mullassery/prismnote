class Prismnote < Formula
  desc "Modern, open-source Jupyter-compatible data science notebook"
  homepage "https://github.com/Mullassery/prismnote"
  license "MIT"

  url "https://files.pythonhosted.org/packages/prismnote-0.3.0-cp313-cp313-macosx_10_12_x86_64.whl"
  sha256 "UPDATE_WITH_ACTUAL_SHA256"

  depends_on "python@3.11"

  def install
    venv = virtualenv_create(libexec, "python3.11")
    venv.pip_install "prismnote==0.3.0"
    bin.install_symlink libexec/"bin/prismnote"
  end

  test do
    system bin/"prismnote", "--version"
  end
end
