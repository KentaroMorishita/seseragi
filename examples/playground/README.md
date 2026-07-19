# Playground samples

このdirectoryは、Seseragiを学ぶ順序から設計したPlayground専用のsourceです。
compilerの回帰確認に使う`examples/spec/artifacts/`とは役割を分けています。

## Learning path

- 初級: 値、関数、ADT、Collectionから実行できるprogramを作る
- 中級: Record、Struct、newtype、Effectでdomainの境界を設計する
- 上級: Monad、Trait、instance、演算子、Signalの抽象化を合成する
- 実践: Signal、HTML、Style、DOMを一つのappへ統合する

sample数は固定しません。各sourceが一つの明確な学習成果を持ち、前の段階で得た
知識から無理なく読めることを優先します。commentは構文を言い換えるためではなく、
コードだけでは分かりにくい型や実行上の関係を説明する場合に限って置きます。

Playgroundのintegration testは全sourceを実際のRust compilerでcompileし、interactive
sample以外はbrowser runtimeで期待する出力まで確認します。
