# Seseragi サンプルコード集

このディレクトリには、Seseragi言語の機能を示すサンプルコードが含まれています。

## ファイル構成

```
examples/
├── README.md                    # このファイル
├── compiled/                    # コンパイル済みファイル
├── 01-advanced-features.ssrg    # 高度な機能
├── 02-tutorial.ssrg             # 🌟 基本チュートリアル（推奨開始点）
├── 03-maybe-monad.ssrg          # Maybe型と演算子
├── 04-either-monad.ssrg         # Either型と演算子
├── 05-list-operations.ssrg      # List型の基本操作
├── 06-list-syntax-sugar.ssrg    # List型シンタックスシュガー
├── 07-show-function.ssrg        # show関数による美しい出力
└── 08-array-list-conversion.ssrg # Array↔List相互変換
```

## サンプルファイル

### 🌟 `02-tutorial.ssrg` - 基本チュートリアル（推奨開始点）
言語の基本機能を一通り学べるチュートリアル
- ✅ 基本型（Int, Float, String, Bool）
- ✅ 変数定義（let）
- ✅ 関数定義（引数あり・なし）
- ✅ 条件分岐（if-then-else）
- ✅ Maybe型の基本
- ✅ Either型の基本

### 🚀 `01-advanced-features.ssrg` - 高度な機能
将来実装予定の高度な機能
- 🚧 パターンマッチング
- 🚧 カスタム型定義
- 🚧 モジュールシステム

### 📖 `03-maybe-monad.ssrg` - Maybe型演算子
Maybe型の演算子使用例
- ✅ ファンクター（`<$>`）の使用方法
- ✅ アプリカティブ（`<*>`）の使用方法
- ✅ モナド（`>>=`）の使用方法
- ✅ 安全な計算チェーン

### 📖 `04-either-monad.ssrg` - Either型演算子
Either型の演算子使用例
- ✅ エラーハンドリングパターン
- ✅ ファンクター（`<$>`）の使用方法
- ✅ アプリカティブ（`<*>`）の使用方法
- ✅ モナド（`>>=`）の使用方法

### 📋 `05-list-operations.ssrg` - List型の基本操作
List型の基本的な操作方法
- ✅ Cons/Emptyコンストラクタ
- ✅ パターンマッチング
- ✅ 再帰的リスト処理
- ✅ List型の関数型操作

### ✨ `06-list-syntax-sugar.ssrg` - List型シンタックスシュガー
美しいList構文の使用例
- ✅ `` `[1, 2, 3] `` シンタックスシュガー
- ✅ `a : b` CONS演算子（右結合）
- ✅ Array型との区別
- ✅ 混合構文の活用

### 🎨 `07-show-function.ssrg` - show関数による美しい出力
開発体験を向上させる美しい出力機能
- ✅ `show`関数による自動整形出力
- ✅ `print`との比較デモ
- ✅ Maybe/Either/List型の美しい表示
- ✅ 複雑なネストデータの整形

### 🔄 `08-array-list-conversion.ssrg` - Array↔List相互変換
実用的なデータ変換パターン
- ✅ `arrayToList`による配列→リスト変換
- ✅ `listToArray`によるリスト→配列変換
- ✅ 空配列の処理
- ✅ 文字列配列の変換

## 実行方法

### 🚀 直接実行（推奨）
```bash
# 基本チュートリアル
seseragi run examples/02-tutorial.ssrg

# Maybe型演算子サンプル
seseragi run examples/03-maybe-monad.ssrg

# Either型演算子サンプル
seseragi run examples/04-either-monad.ssrg

# List型基本操作
seseragi run examples/05-list-operations.ssrg

# List型シンタックスシュガー
seseragi run examples/06-list-syntax-sugar.ssrg

# show関数による美しい出力
seseragi run examples/07-show-function.ssrg

# Array↔List相互変換
seseragi run examples/08-array-list-conversion.ssrg

# 高度な機能
seseragi run examples/01-advanced-features.ssrg
```

### 🔄 ファイル監視で実行
ファイルを変更するたびに自動実行したい場合：
```bash
# ファイル監視で実行（開発中に便利）
seseragi run examples/02-tutorial.ssrg --watch
# or 短縮形
seseragi run examples/02-tutorial.ssrg -w
```

### 📂 コンパイル後実行
```bash
# コンパイル（シンプル版）
seseragi examples/02-tutorial.ssrg  # examples/02-tutorial.ts に出力

# コンパイル（出力先指定）
seseragi examples/02-tutorial.ssrg -o examples/compiled/tutorial.ts

# 実行
bun examples/02-tutorial.ts
```

## 学習の進め方

1. **`02-tutorial.ssrg`から開始** - 基本概念を理解
   ```bash
   seseragi run examples/02-tutorial.ssrg
   ```

2. **`03-maybe-monad.ssrg`でMaybe型を学習** - 安全な計算方法を理解
   ```bash
   seseragi run examples/03-maybe-monad.ssrg
   ```

3. **`04-either-monad.ssrg`でEither型を学習** - エラーハンドリング方法を理解
   ```bash
   seseragi run examples/04-either-monad.ssrg
   ```

4. **`01-advanced-features.ssrg`で将来機能を確認** - 言語の方向性を理解
   ```bash
   seseragi run examples/01-advanced-features.ssrg
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

✅ **Array↔List変換**
- `arrayToList` - Array→List変換
- `listToArray` - List→Array変換

✅ **標準出力**
- `print` - 値の出力
- `show` - 美しい整形出力（開発体験向上）

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