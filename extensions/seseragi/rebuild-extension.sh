#!/bin/bash
# VSCode拡張機能再ビルドスクリプト

set -e

echo "🔧 Rebuilding Seseragi VSCode Extension..."

# 拡張機能再ビルド
echo "📦 Compiling TypeScript..."
bun run compile

# 古いパッケージ削除 & 新パッケージ作成
echo "🗑️  Removing old packages..."
rm -f *.vsix

echo "📦 Creating new package..."
vsce package

# 古い拡張機能をアンインストール
echo "🗂️  Uninstalling old extension..."
code --uninstall-extension seseragi-dev.seseragi-language-support 2>/dev/null || true

# 新しい拡張機能をインストール
echo "⬇️  Installing new extension..."
VSIX_FILE=$(ls seseragi-language-support-*.vsix | head -1)
code --install-extension "$VSIX_FILE"

echo "✅ Extension rebuilt and installed successfully!"
echo "🔄 Please reload VSCode window: Cmd+Shift+P → 'Developer: Reload Window'"