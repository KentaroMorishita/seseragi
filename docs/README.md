# Seseragi 言語仕様

このディレクトリは Seseragi 言語の正本です。

仕様は、特定のコンパイラ実装、出力言語、ランタイムライブラリに依存しません。
実装がこの文書と異なる場合は、実装の挙動ではなくこの文書を基準にします。

## 仕様の読み方

- [仕様索引・feature map・cross-cutting invariants](./spec/README.md)
- [仕様カバレッジと未定義項目](./SPEC_COVERAGE.md)
- [言語の定義と設計原則](./spec/00-language.md)
- [字句・構文・演算子](./spec/01-syntax.md)
- [型システム](./spec/02-types.md)
- [データ・式・パターン](./spec/03-data-and-expressions.md)
- [型クラスとdo notation](./spec/04-type-classes.md)
- [失敗・Effect・Signal](./spec/05-effects.md)
- [Seseragiモジュール](./spec/06-modules-and-interop.md)
- [TypeScript interop](./spec/07-typescript-interop.md)
- [`.d.ts` からのbinding生成](./spec/08-dts-conversion.md)
- [標準ライブラリの契約](./spec/09-standard-library.md)
- [標準ライブラリsurface](./spec/10-library-surface.md)
- [Appendix A: 文法要約](./spec/grammar.md)

## 規範性

本文中の「〜である」「〜しなければならない」は規範です。「例」は規範を
説明するコードであり、例だけから別の規則を推測してはなりません。

仕様に書かれていない構文や暗黙変換は存在しません。将来の拡張候補はこの正本へ
混ぜず、採用時に意味・型付け規則・失敗条件を同時に追加します。
