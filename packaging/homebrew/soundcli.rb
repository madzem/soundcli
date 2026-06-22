# Linuxbrew formula (soundcli needs MPRIS/D-Bus + PipeWire; it does not run on macOS).
# Publish via a `madzem/homebrew-soundcli` tap; set `sha256` from the release tarball:
#   curl -sL https://github.com/madzem/soundcli/archive/refs/tags/v1.4.0.tar.gz | sha256sum
class Soundcli < Formula
  desc "Terminal remote + now-playing dashboard for SoundCloud (controls your browser over MPRIS)"
  homepage "https://github.com/madzem/soundcli"
  url "https://github.com/madzem/soundcli/archive/refs/tags/v1.4.0.tar.gz"
  sha256 "REPLACE_WITH_SOURCE_TARBALL_SHA256"
  license "MIT"
  head "https://github.com/madzem/soundcli.git", branch: "main"

  depends_on :linux # needs MPRIS / D-Bus + PipeWire; cannot run on macOS
  depends_on "pkg-config" => :build
  depends_on "rust" => :build
  depends_on "dbus"

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "soundcli", shell_output("#{bin}/soundcli --help")
  end
end
