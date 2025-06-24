# Seseragi サンプルコード集

このディレクトリには、Seseragi言語の様々な機能を示すサンプルコードが含まれています。

## ディレクトリ構造

```
examples/
├── README.md                    # このファイル
├── compiled/                    # コンパイル済みファイル（.gitignoreで除外）
├── working-samples.ssrg         # ✅ 動作確認済みメインサンプル
├── functional-style.ssrg        # 関数型スタイル構文デモ
├── tutorial.ssrg               # 段階的学習チュートリアル
├── basic-samples.ssrg          # 基本機能紹介
└── advanced-features.ssrg      # 高度な機能と将来機能
```

## サンプルファイル

### 🌟 `working-samples.ssrg` - メインサンプル（推奨）
動作確認済みの包括的なサンプル
- 基本的な値と関数
- カリー化と部分適用
- 条件分岐と再帰関数
- 関数型スタイル構文
- **完全にテスト済み** ✅

### 📖 `tutorial.ssrg` - 学習チュートリアル
初心者向けの段階的ガイド
- Step 1: 基本的な値と出力
- Step 2: 関数の定義と呼び出し
- Step 3: カリー化された関数
- Step 4: 条件分岐
- Step 5: 文字列操作
- Step 6: 再帰関数
- Step 7: 関数型スタイル

### 🎯 `functional-style.ssrg` - 関数型構文デモ
関数型プログラミングの核となる構文
- 括弧なし関数呼び出し: `print "hello"`
- ビルトイン関数の使用
- 変数との組み合わせ

### 📚 `basic-samples.ssrg` - 基本機能集
Seseragiの主要機能を網羅
- 基本型（Int, Float, Bool, String）
- 標準出力関数
- 関数定義とカリー化
- 条件分岐と文字列操作

### 🚀 `advanced-features.ssrg` - 高度な機能
現在の高度な例と将来実装予定の機能
- 現在実装済みの高度な例
- 将来実装予定の機能（コメント形式）
- パターンマッチング、Maybe型、パイプライン等

## サンプルの実行方法

### 基本的な使用法
```bash
# メインサンプルの実行（推奨）
seseragi compile examples/working-samples.ssrg --output examples/compiled/working-samples.ts
bun examples/compiled/working-samples.ts

# チュートリアルの実行
seseragi compile examples/tutorial.ssrg --output examples/compiled/tutorial.ts
bun examples/compiled/tutorial.ts
```

### 便利なワンライナー
```bash
# コンパイル＆実行
seseragi compile examples/working-samples.ssrg --output examples/compiled/working-samples.ts && bun examples/compiled/working-samples.ts

# 複数サンプルを一括実行
for file in examples/*.ssrg; do
  name=$(basename "$file" .ssrg)
  seseragi compile "$file" --output "examples/compiled/${name}.ts" && bun "examples/compiled/${name}.ts"
  echo "--- $name completed ---"
done
```

### その他のオプション
```bash
# ファイル監視モード
seseragi compile examples/tutorial.ssrg --output examples/compiled/tutorial.ts --watch

# 標準出力への出力（デバッグ用）
seseragi compile examples/functional-style.ssrg --output -
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