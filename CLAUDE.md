# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## プロジェクト概要

**Seseragi**: TypeScriptにトランスパイルするプログラミング言語
字句解析 → 構文解析 → 型推論 → コード生成のパイプラインを持つ

## 日常コマンド

```bash
bun run all     # 全部込み（開発時推奨）
bun run build   # ビルドのみ
bun run check   # 品質チェック（format + lint + typecheck + test）
bun run vscode  # VS Code拡張リビルド
bun run test    # テスト実行
```

## 開発ルール

### 言語使い分け
- **Claude とのやりとり**: 日本語（最重要）
- **コード内コメント**: 日本語

**GitHub:**
- **Issue**: タイトル英語、内容日本語
- **Commit**: 英語（Conventional Commits形式）
- **Pull Request**: 英語（タイトル・説明）

### Git運用
```bash
# ✅ 正しいフロー
git checkout -b feature/new-feature  # 必ずfeatureブランチ
# 開発・テスト
bun run all                          # 品質チェック
git commit -m "feat: description"    # conventional commits
git push -u origin feature/new-feature
gh pr create                         # PR作成

# ❌ 禁止事項
# - mainブランチ直接編集
# - mainブランチ直接push
```

## 🚨 頻出トラブル対処法

### 1. 新演算子で型推論エラー
**症状**: `Unknown binary operator` エラー
**原因**: `src/type-inference.ts`の型推論ルール追加忘れ
**対処**: 新演算子の型推論ルールを必ず追加

### 2. VS Code構文ハイライト未更新
**症状**: 新構文が灰色のまま
**原因**: 拡張機能再ビルド忘れ
**対処**: `bun run vscode` + VS Code完全再起動

### 3. LSPサーバー情報が古い
**症状**: エラー表示が古い、型情報が間違い
**原因**: `dist/`未更新
**対処**: `bun run build`でdist/更新

## 言語機能追加フロー

新しい構文・演算子追加時の確実な手順：

```bash
# 1. テスト駆動開発
vi tests/new-feature.test.ts         # 先にテストケース作成

# 2. コア実装（必ずこの順番）
vi src/lexer.ts                      # トークン追加
vi src/parser.ts                     # パーサー更新
vi src/ast.ts                        # AST拡張
vi src/type-inference.ts             # 🚨型推論ルール（最重要）
vi src/codegen.ts                    # TypeScript生成

# 3. VS Code拡張更新
vi extensions/seseragi/syntaxes/seseragi.tmLanguage.json  # 構文ハイライト
vi extensions/seseragi/package.json                       # バージョンアップ

# 4. 全体チェック
bun run all                          # ビルド・テスト・品質チェック

# 5. VS Code動作確認
# VS Code完全再起動 → 構文ハイライト確認
```

## 開発マインドセット

- **一機能ずつ**: 複数機能同時実装禁止
- **テストファースト**: 実装前にテストケース作成
- **型推論は忘れやすい**: 新演算子は必ず型推論ルール追加
- **小さくコミット**: 動く状態で頻繁にコミット
- **毎回全体テスト**: `bun run all`でチェック

## 技術スタック

```
アーキテクチャ:
src/
├── lexer.ts          # 字句解析
├── parser.ts         # 構文解析（手書き再帰降下）
├── type-inference.ts # Hindley-Milner型推論（🚨重要）
├── codegen.ts        # TypeScript生成
└── lsp/              # Language Server

技術:
- Runtime: Bun
- Test: Bunテストランナー
- Quality: Biome（lint + format）
- CLI: Commander.js
- LSP: vscode-languageserver
```

## セットアップ

```bash
bun install
bun run all  # 初回ビルド + VS Code拡張インストール
```