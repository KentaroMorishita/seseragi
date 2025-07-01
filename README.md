# Seseragi Programming Language

Seseragiは、TypeScriptにトランスパイルされるプログラミング言語です。

## 特徴

- **副作用の明示的管理** - 安全で予測可能なコード
- **静的型付け** - 型安全性の保証
- **カリー化された関数** - デフォルトで部分適用をサポート
- **モナド合成** - Maybe、Either、IOモナドによる安全な処理
- **パイプライン演算子** - 関数合成の直感的な記述

## クイックスタート

### 依存関係のインストール

```bash
bun install
```

### Seseragiコンパイラの実行

```bash
# TypeScriptトランスパイラのテスト
bun run src/main.ts

# 全テスト実行
bun test

# 型チェック
bun run typecheck
```

### サンプルコード

Seseragi言語（`.ssrg`）のサンプル：

```seseragi
// 基本的な関数定義
fn add a: Int -> b: Int -> Int = a + b

// 関数適用演算子の使用例
fn double x: Int -> Int = x * 2

// Maybe型による安全な処理
let someValue = Just 42
let nothingValue = Nothing

// 関数適用演算子（$）
let result = toString $ add 5 3

// 実行例
print $ add 10 5     // 15
print $ double 7     // 14
print someValue      // { tag: "Just", value: 42 }
print result         // "8"
```

## Seseragiコマンドライン

### インストール

```bash
# 依存関係をインストール
bun install

# 開発用ビルド
bun run build

# seseragiコマンドをグローバルにインストール（オプション）
npm link
```

### コマンド一覧

#### `seseragi run` - 直接実行

Seseragiファイルを直接実行します：

```bash
# 基本的な実行
seseragi run examples/tutorial.ssrg

# デバッグ用（一時ファイルを保持）
seseragi run examples/tutorial.ssrg --keep-temp

# カスタム一時ディレクトリ指定
seseragi run examples/tutorial.ssrg --temp-dir ./tmp
```

#### `seseragi compile` - TypeScriptコンパイル

SeseragiファイルをTypeScriptにコンパイルします：

```bash
# 基本的なコンパイル
seseragi compile input.ssrg --output output.ts

# ファイル監視モード
seseragi compile input.ssrg --output output.ts --watch

# 関数宣言スタイル使用
seseragi compile input.ssrg --output output.ts --function-declarations

# コメントなしで出力
seseragi compile input.ssrg --output output.ts --no-comments

# ランタイムモード指定
seseragi compile input.ssrg --output output.ts --runtime embedded
seseragi compile input.ssrg --output output.ts --runtime import
seseragi compile input.ssrg --output output.ts --runtime minimal
```

#### `seseragi format` - コードフォーマット

Seseragiファイルをフォーマットします：

```bash
# ファイルをフォーマット（標準出力）
seseragi format input.ssrg

# インプレース編集
seseragi format input.ssrg --in-place

# フォーマットチェック
seseragi format input.ssrg --check

# 出力ファイル指定
seseragi format input.ssrg --output formatted.ssrg
```

### 使用例

#### 開発ワークフロー

```bash
# 1. Seseragiファイルを作成
echo 'print "Hello, Seseragi!"' > hello.ssrg

# 2. コードをフォーマット
seseragi format hello.ssrg --in-place

# 3. 直接実行して動作確認
seseragi run hello.ssrg

# 4. TypeScriptにコンパイル
seseragi compile hello.ssrg --output hello.ts

# 5. 生成されたTypeScriptを確認
cat hello.ts
```

#### 継続的開発

```bash
# ファイル監視モードでコンパイル
seseragi compile src/main.ssrg --output dist/main.ts --watch

# 別ターミナルで実行テスト
seseragi run src/main.ssrg
```

### ランタイムモードの説明

- **`minimal`** (デフォルト): 必要な機能のみを含む最小ランタイム
- **`embedded`**: 全機能を含む埋め込み式ランタイム  
- **`import`**: 外部ランタイムライブラリからインポート

### 出力例

```bash
$ seseragi run hello.ssrg
Parsing hello.ssrg...
Generating TypeScript code...
Running...

Hello, Seseragi!

$ seseragi compile hello.ssrg --output hello.ts
Parsing hello.ssrg...
Generating TypeScript code...
✓ Compiled to hello.ts
```

### モナド演算子の例

```seseragi
fn double x: Int -> Int = x * 2
fn add a: Int -> b: Int -> Int = a + b
fn increment x: Int -> Maybe<Int> = Just (x + 1)

// ファンクター演算子 (<$>)
let doubled = double <$> Just 21    // Just 42

// アプリカティブ演算子 (<*>)
let added = Just add <*> Just 10 <*> Just 5    // Just 15

// モナド演算子 (>>=)
let chained = Just 20 >>= increment    // Just 21
```

## VS Code拡張

Seseragi言語用のVS Code拡張が含まれています：

### 機能
- ✅ シンタックスハイライト
- ✅ 括弧の自動補完
- ✅ コメントサポート
- ✅ 言語固有の設定

### セットアップ
1. `.ssrg` 拡張子のファイルを作成
2. 自動的にシンタックスハイライトが適用される
3. サンプル: `examples/sample.ssrg`

## 開発情報

### 技術スタック
- **言語**: TypeScript/JavaScript
- **パーサー**: 手書き再帰降下パーサー
- **テスト**: Bun Test
- **ビルド**: Bun
- **ランタイム**: [Bun](https://bun.sh)

### プロジェクト構造
```
src/
├── lexer.ts      # 字句解析器
├── parser.ts     # 構文解析器
├── ast.ts        # 抽象構文木定義
├── codegen.ts    # TypeScriptコード生成
└── main.ts       # メインエントリーポイント

tests/            # テストファイル
examples/         # Seseragiサンプルコード
.vscode/          # VS Code拡張とワークスペース設定
```

### 開発コマンド
```bash
# 開発サーバー
bun run dev

# ビルド
bun run build:all

# テスト
bun test
bun run test:watch

# 品質チェック
bun run typecheck
bun run lint
bun run format
```

## コントリビューション

詳細な開発ガイドは `CLAUDE.md` を参照してください。

## ライセンス

MIT License
