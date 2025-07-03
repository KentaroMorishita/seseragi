# Seseragi Extensions

Seseragiプログラミング言語の開発ツールと拡張機能です。

## VS Code拡張 (seseragi)

Seseragi言語用のVS Code拡張機能。シンタックスハイライト、LSPサーバー統合、自動フォーマットを提供します。

### 機能

- ✅ **シンタックスハイライト** - 全言語要素をサポート
- ✅ **Language Server Protocol** - リアルタイム型チェック・エラー診断
- ✅ **自動フォーマット** - 保存時コード整形
- ✅ **型情報表示** - Hindley-Milner型推論結果をホバー表示

### クイックスタート

```bash
# プロジェクトルートで実行
bun run build

# VS Code再起動後、.ssrgファイルで拡張機能が有効化
```

### 開発

```bash
# 拡張機能のみ再ビルド
./scripts/vscode.sh

# 手動ビルド
cd extensions/seseragi
bun run compile
vsce package
code --install-extension seseragi-extension.vsix
```

## ディレクトリ構造

```
extensions/
├── README.md                    # この文書  
└── seseragi/                    # VS Code拡張機能
    ├── package.json             # 拡張機能設定
    ├── syntaxes/                # TextMate文法定義
    ├── src/extension.ts         # 拡張機能エントリーポイント
    └── README.md                # 拡張機能詳細
```

## ライセンス

Apache-2.0