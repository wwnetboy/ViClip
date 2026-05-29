cask "viclip" do
  version "1.0.8"
  sha256 "2ac2bb535d05266347e80f05f5b307f25018a9097c6de4631309fcd530236227"

  url "https://github.com/wwnetboy/ViClip/releases/download/v#{version}/ViClip.app.zip"
  name "ViClip"
  desc "Lightweight desktop productivity suite — clipboard manager, quick phrases, translation"
  homepage "https://github.com/wwnetboy/ViClip"

  # macOS Gatekeeper: unsigned app requires user to allow manually after install.
  # Homebrew Cask handles this by placing the app in /Applications and
  # removing the quarantine attribute automatically.

  app "ViClip.app"

  zap trash: [
    "~/Library/Application Support/com.viclip.app",
    "~/Library/Preferences/com.viclip.app.plist",
    "~/Library/Saved Application State/com.viclip.app.savedState",
  ]
end
