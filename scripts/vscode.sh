#!/bin/bash
set -e

cd extensions/seseragi

echo "🔨 Building & installing VS Code extension..."

# Clean, build, package with fixed name
rm -f *.vsix
bun run compile
yes | bunx vsce package --out seseragi-extension.vsix

# Reinstall
code --uninstall-extension seseragi-dev.seseragi-language-support || true
code --install-extension seseragi-extension.vsix

echo "✅ Extension installed! Reload VS Code window."