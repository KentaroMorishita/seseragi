# Runnable sample catalog

このdirectoryは、現行Rust版Seseragiでcompile・実行できる人向けsampleの正本です。
PlaygroundとCLI sample checkは同じ`main.ssrg`を読みます。compilerの最小回帰caseは
`examples/spec/artifacts/`、完成言語仕様の設計教材は`examples/spec/lessons/`に分離します。

## sampleを追加する

stable slugのdirectoryを一つ追加し、次を置きます。

```text
examples/samples/<sample-id>/
  main.ssrg
  sample.json
  guide.md
  stdin.txt       # stdinを使う場合だけ
  stdout.txt      # interactive以外
```

`sample.json`は[`sample.schema.json`](./sample.schema.json)に従います。学習順はsample
metadataへ持たせず、[`learning-paths.json`](./learning-paths.json)で定義します。一つのsampleを
複数pathから参照できます。

```sh
cd apps/playground
bun run samples:generate
```

generatorはdirectoryを自動発見し、metadata schema、ID重複、source/output欠落、前提graphの
循環、学習path参照を検査します。生成manifestはsource hashを持ち、`samples:check`がstaleを
検出します。中央のimport一覧やID対応表は手で編集しません。

`stdout.txt`はbrowser/CLI hostが返すstdoutとbyte単位で一致させ、不要な末尾newlineを
追加しません。説明はsourceへ大量に埋め込まず`guide.md`へ書きます。
