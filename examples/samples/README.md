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

## Web UI source readability contract

HTML sampleのsource自体を、利用者がコピーできるcanonical exampleとして扱います。基本のsection順は
`import`、domainの`type` / `struct`、`update`、style helper、画面上の意味単位のcomponent、
`mount` / `main`です。説明は長いinline commentへ詰めず、関数名・型と`guide.md`へ分けます。

### utility class

1〜4 tokenで80文字以内の局所的な`className`は直接のliteralで構いません。5 token以上、または
propertyを含む行が80文字を超える場合は、sample内の実在する`cx` helperとnamed valueへ分けます。
`cx [...]`は一行一tokenにし、formatter後も縦の配列をcanonical sourceとして保持します。

```seseragi
fn cx classes: Array<String> -> String =
  join " " classes

let cardClass =
  cx [
    "rounded-2xl",
    "bg-white",
    "p-6",
    "shadow-lg"
  ]
```

single-file sampleは`cx`を同じsource内へ定義します。multi-file sampleはworkspaceの
`styles.ssrg`から`pub fn cx`を公開し、利用moduleが明示importします。これによりExplorerから
helperの実装へ辿れます。巨大なtemplate literalで動的classを組み立てず、状態ごとの意味を持つ
named class valueを`match` / `if`で選びます。

### `html.Style`

utility contractにない値、動的な色・幅・transition、CSS custom propertyのような意味を持つ
visual tokenには`html.style`を使います。同じstyle objectを二回以上使う場合、またはelement内で
複数propertyを持つ場合は、`heroStyle`、`progressStyle mode`のようなnamed value / functionへ
抽出します。URLを含むsecurity-sensitive attributeはstyleへ逃がさず、`WebUrl`の公開契約を使います。

### component boundary

page全体を一つの巨大な`view`へ置かず、hero、form、summary、item list、empty state、action groupなど
画面上の意味単位へ分けます。単なる一行wrapperは増やさず、propsの意味が関数名と型から読める境界を
選びます。`samples:check`は長大な直接`className`、横に圧縮された大きな`cx`配列、所在が追えない
`cx` helperを拒否します。

```sh
cd apps/playground
bun run samples:generate
```

generatorはdirectoryを自動発見し、metadata schema、ID重複、source/output欠落、前提graphの
循環、Discover groupのkind・参照・一意配置を検査します。生成manifestはsource hashを持ち、
`samples:check`がstaleを検出します。中央のimport一覧やID対応表は手で編集しません。

`stdout.txt`はbrowser/CLI hostが返すstdoutとbyte単位で一致させ、不要な末尾newlineを
追加しません。説明はsourceへ大量に埋め込まず`guide.md`へ書きます。
