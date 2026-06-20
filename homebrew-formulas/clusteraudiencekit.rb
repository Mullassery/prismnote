class Clusteraudiencekit < Formula
  desc "High-performance audience segmentation engine with Python bindings"
  homepage "https://github.com/Mullassery/clusteraudiencekit"
  license "MIT"

  url "https://files.pythonhosted.org/packages/clusteraudiencekit-0.1.0-cp313-cp313-macosx_10_12_x86_64.whl"
  sha256 "UPDATE_WITH_ACTUAL_SHA256"

  depends_on "python@3.11"

  def install
    venv = virtualenv_create(libexec, "python3.11")
    venv.pip_install "clusteraudiencekit==0.1.0"
  end

  test do
    system "python3", "-c", "import clusteraudiencekit; print(clusteraudiencekit.__version__)"
  end
end
