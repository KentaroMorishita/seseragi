#!/bin/bash
set -e

echo "🔨 Building VS Code extension..."

# Move to extension directory
cd extensions/seseragi

# Clean old packages
echo "🧹 Cleaning old packages..."
rm -f *.vsix

# Build the extension
echo "🏗️ Compiling TypeScript..."
bun run compile

# Package the extension
echo "📦 Creating VSIX package..."
# Auto-confirm prompts with yes
yes | bunx vsce package

# Find the generated vsix file
VSIX_FILE=$(ls *.vsix 2>/dev/null | head -n 1)

if [ -z "$VSIX_FILE" ]; then
  echo "❌ Error: No VSIX file generated"
  exit 1
fi

echo "✅ Package created: $VSIX_FILE"

# Uninstall old extension
echo "🗑️ Uninstalling old extension..."
code --uninstall-extension seseragi-dev.seseragi-language-support || true

# Install new extension
echo "📥 Installing new extension..."
code --install-extension "$VSIX_FILE"

echo "🎉 VS Code extension rebuild complete!"
echo "💡 Reload VS Code window to activate the changes"