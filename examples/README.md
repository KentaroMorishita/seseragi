# Seseragi サンプルコード集

このディレクトリには、Seseragi言語の機能を示すサンプルコードが含まれています。

## ファイル構成

```
examples/
├── README.md              # このファイル
├── compiled/              # コンパイル済みファイル
├── tutorial.ssrg          # 🌟 基本チュートリアル（推奨開始点）
├── maybe-example.ssrg     # Maybe型と演算子
├── either-example.ssrg    # Either型と演算子
└── advanced-features.ssrg # 高度な機能
```

## サンプルファイル

### 🌟 `tutorial.ssrg` - 基本チュートリアル（推奨開始点）
言語の基本機能を一通り学べるチュートリアル
- ✅ 基本型（Int, Float, String, Bool）
- ✅ 変数定義（let）
- ✅ 関数定義（引数あり・なし）
- ✅ 条件分岐（if-then-else）
- ✅ Maybe型の基本
- ✅ Either型の基本

### 📖 `maybe-example.ssrg` - Maybe型演算子
Maybe型の演算子使用例
- ✅ ファンクター（`<$>`）の使用方法
- ✅ アプリカティブ（`<*>`）の使用方法
- ✅ モナド（`>>=`）の使用方法
- ✅ 安全な計算チェーン

### 📖 `either-example.ssrg` - Either型演算子
Either型の演算子使用例
- ✅ エラーハンドリングパターン
- ✅ ファンクター（`<$>`）の使用方法
- ✅ アプリカティブ（`<*>`）の使用方法
- ✅ モナド（`>>=`）の使用方法

### 🚀 `advanced-features.ssrg` - 高度な機能
将来実装予定の高度な機能
- 🚧 パターンマッチング
- 🚧 カスタム型定義
- 🚧 モジュールシステム

## 実行方法

### 🚀 直接実行（推奨）
```bash
# 基本チュートリアル
seseragi run examples/tutorial.ssrg

# Maybe型演算子サンプル
seseragi run examples/maybe-example.ssrg

# Either型演算子サンプル
seseragi run examples/either-example.ssrg

# 高度な機能
seseragi run examples/advanced-features.ssrg
```

### 📂 コンパイル後実行
```bash
# コンパイル
seseragi compile examples/tutorial.ssrg --output examples/compiled/tutorial.ts

# 実行
bun examples/compiled/tutorial.ts
```

## 学習の進め方

1. **`tutorial.ssrg`から開始** - 基本概念を理解
   ```bash
   seseragi run examples/tutorial.ssrg
   ```

2. **`maybe-example.ssrg`でMaybe型を学習** - 安全な計算方法を理解
   ```bash
   seseragi run examples/maybe-example.ssrg
   ```

3. **`either-example.ssrg`でEither型を学習** - エラーハンドリング方法を理解
   ```bash
   seseragi run examples/either-example.ssrg
   ```

4. **`advanced-features.ssrg`で将来機能を確認** - 言語の方向性を理解
   ```bash
   seseragi run examples/advanced-features.ssrg
   ```

## 実装済み機能

✅ **基本機能**
- 基本型（Int, Float, Bool, String）
- 変数宣言（let）
- 関数定義（fn）
- 条件分岐（if-then-else）

✅ **関数型プログラミング**
- カリー化された関数
- 部分適用
- 高階関数

✅ **モナド型**
- Maybe型（Just, Nothing）
- Either型（Left, Right）

✅ **モナド演算子**
- ファンクター：`<$>` - `double <$> Just 42`
- アプリカティブ：`<*>` - `Just add <*> Just 5 <*> Just 3`
- モナド：`>>=` - `Just 10 >>= safeDivide 2`

✅ **標準出力**
- `print` - 値の出力

## 将来実装予定機能

🚧 **型システム拡張**
- カスタム代数的データ型
- List型とArray型
- 型推論の改善

🚧 **パターンマッチング**
- match式
- ガード式
- 網羅性チェック

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
   → 適切な型変換を行ってください

### デバッグのコツ

1. **小さなコードから始める** - 複雑な式を分割して確認
2. **型注釈を明示する** - 明示的に型を指定
3. **段階的に構築する** - 機能を少しずつ追加して動作確認