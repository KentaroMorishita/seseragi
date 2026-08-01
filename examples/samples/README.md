# Runnable sample catalog

このdirectoryは、現行Rust版Seseragiでcompile・実行できる人向けsampleの正本です。
PlaygroundとCLI sample checkは同じ`main.ssrg`を読みます。compilerの最小回帰caseは
`examples/spec/artifacts/`、完成言語仕様の設計教材は`examples/spec/lessons/`に分離します。

## sampleを追加する

stable slugのdirectoryを一つ追加し、次を置きます。

```text
examples/samples/<sample-id>/
  main.ssrg
  feature/helper.ssrg # project sampleの場合
  sample.json
  guide.md
  stdin.txt       # stdinを使う場合だけ
  stdout.txt      # interactive以外
```

複数fileのproject sampleは`sample.json`へ`workspace`を追加し、entry、
読み込むfile、最初に開くtab、展開するfolderを宣言します。`files.source`は
`workspace.entry`と同じpathにします。

```json
{
  "files": {
    "source": "main.ssrg",
    "guide": "guide.md",
    "expectedOutput": "stdout.txt"
  },
  "workspace": {
    "entry": "main.ssrg",
    "files": ["main.ssrg", "feature/helper.ssrg"],
    "active": "main.ssrg",
    "open": ["main.ssrg", "feature/helper.ssrg"],
    "expanded": ["feature"]
  }
}
```

`sample.json`は[`sample.schema.json`](./sample.schema.json)に従います。基礎学習の正本は
[`../tour/`](../tour/)の14 lessonです。通常PlaygroundのDiscoverへ出すRecipe / Showcaseは
[`discover-groups.json`](./discover-groups.json)へ目的別に一度だけ配置します。Tour作成の根拠として
保持する`lesson` sampleはDiscoverへ表示しません。

`outputMode: "html"`のsampleは、Previewが注入するutility CSSと`className`のtokenを
`samples:check`で照合します。静的なclassは直接の文字列または`cx [...]`へ置きます。
任意式が返すutilityは`preview.dynamicUtilities`、見た目を持たないsemantic classは
`preview.customClasses`へtoken単位で宣言します。custom classへCSSは追加されないため、
固有の視覚値は`html.Style`を使います。

```sh
cd apps/playground
bun run samples:generate
```

generatorはdirectoryを自動発見し、metadata schema、ID重複、source/output欠落、前提graphの
循環、Discover groupのkind・参照・一意配置を検査します。生成manifestはsource hashを持ち、
`samples:check`がstaleを検出します。中央のimport一覧やID対応表は手で編集しません。

`stdout.txt`はbrowser/CLI hostが返すstdoutとbyte単位で一致させ、不要な末尾newlineを
追加しません。説明はsourceへ大量に埋め込まず`guide.md`へ書きます。
