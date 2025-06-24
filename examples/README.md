# Seseragi サンプルコード集

このディレクトリには、Seseragi言語の様々な機能を示すサンプルコードが含まれています。

## サンプルファイル

### 1. `functional-style.ssrg`
関数型スタイルの基本的な構文を示すサンプル
- 括弧なし関数呼び出し
- ビルトイン関数の使用
- 基本的な式の構成

### 2. `tutorial.ssrg` 
初心者向けの段階的チュートリアル
- 基本的な値と変数
- 関数の定義と呼び出し
- カリー化と部分適用
- 条件分岐
- 再帰関数
- 高階関数

### 3. `basic-samples.ssrg`
Seseragiの基本機能を幅広くカバー
- 基本型の使用
- 標準出力
- 関数定義
- パイプライン演算子
- 型定義

### 4. `advanced-features.ssrg`
高度な機能と将来実装予定の機能
- 複雑な関数合成
- モナド的操作
- パターンマッチング（将来実装）
- カスタム演算子（将来実装）

## サンプルの実行方法

### コンパイルと実行
```bash
# TypeScriptにトランスパイル
seseragi compile examples/tutorial.ssrg

# 生成されたTypeScriptファイルを実行
bun examples/tutorial.ts
```

### ファイル監視モード
```bash
# ファイルの変更を監視して自動コンパイル
seseragi compile examples/tutorial.ssrg --watch
```

### 標準出力への出力
```bash
# コンパイル結果を標準出力に出力
seseragi compile examples/tutorial.ssrg --output -
```

## 学習の進め方

1. **`tutorial.ssrg`から開始** - 基本概念を段階的に学習
2. **`functional-style.ssrg`で構文に慣れる** - 関数型の書き方を理解
3. **`basic-samples.ssrg`で機能を探索** - より多くの機能を試す
4. **`advanced-features.ssrg`で応用を学ぶ** - 高度なパターンを学習

## 現在実装済みの機能

✅ **基本機能**
- 基本型（Int, Float, Bool, String）
- 変数宣言（let）
- 関数定義（fn）
- 条件分岐（if-then-else）

✅ **関数型プログラミング**
- カリー化された関数
- 部分適用
- 高階関数
- 再帰関数

✅ **標準出力**
- `print` - 値の出力
- `putStrLn` - 改行付き出力
- `toString` - 文字列変換

✅ **関数呼び出し構文**
- 括弧付き: `print("hello")`
- 括弧なし: `print "hello"`（関数型スタイル）

## 将来実装予定の機能

🚧 **型システム拡張**
- Maybe型、Either型
- List型とArray型
- カスタム代数的データ型

🚧 **パターンマッチング**
- match式
- ガード式
- 網羅性チェック

🚧 **モナド操作**
- >>= 演算子
- do記法
- IO型

🚧 **モジュールシステム**
- import/export
- 名前空間
- 型エイリアス

## トラブルシューティング

### よくあるエラー

1. **構文エラー**
   ```
   ParseError: Expected ':' after parameter name
   ```
   → 関数パラメータには型注釈が必要です

2. **型エラー**
   ```
   Type mismatch: expected String, got Int
   ```
   → `toString`を使って型を変換してください

3. **未定義関数エラー**
   ```
   Unknown function: undefined_function
   ```
   → 関数が定義されているか確認してください

### デバッグのコツ

1. **小さなコードから始める** - 複雑な式を分割して確認
2. **型注釈を明示する** - 型推論に頼らず明示的に型を指定
3. **段階的に構築する** - 機能を少しずつ追加して動作確認

## コントリビューション

新しいサンプルコードの追加や既存コードの改善は歓迎します！
- わかりやすいコメント
- 段階的な説明
- 実用的な例
- エラーハンドリングの例

を含めてください。