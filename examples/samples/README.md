# サンプル集

現在のコンパイラで実際に動くサンプルを集めたディレクトリです。
Playground の「Discover」とCLI のサンプルチェックは、どちらもここにある同じ `main.ssrg` を読みます。

## サンプルを動かす

- **ブラウザで**: [Playground](https://seseragi.vercel.app/) の Discover から開けます
- **CLI で**: `seseragi run examples/samples/<サンプル名>/main.ssrg`

関連するディレクトリ:

| ディレクトリ | 内容 |
| --- | --- |
| [`../tour/`](../tour/) | 言語を順番に学ぶ Tour のレッスン |
| `../spec/artifacts/` | コンパイラの回帰テスト用の最小ケース |
| `../spec/lessons/` | 言語仕様の説明用プログラム |

---

以下は、サンプルを追加・編集する人向けの説明です。

## サンプルを追加する

サンプル名（後から変えない ID）のディレクトリを作り、次のファイルを置きます。

```text
examples/samples/<sample-id>/
  main.ssrg
  sample.json
  guide.md
  stdin.txt       # 標準入力を使う場合だけ
  stdout.txt      # 対話型でない場合
```

複数ファイルからなるサンプルは、通常の Seseragi パッケージと同じ構成にします。

```text
examples/samples/<sample-id>/
  seseragi.toml
  src/
    main.ssrg
    feature/helper.ssrg
  sample.json
  guide.md
  stdin.txt       # 標準入力を使う場合だけ
  stdout.txt      # 対話型でない場合
```

ファイル構成は `seseragi.toml` の `layout.source` と `run.entry`、そして `src/` 以下のファイルで決まります。
`sample.json` の `workspace` には、Playground で最初に開くタブと展開するフォルダだけを書きます。
エントリやファイル一覧をここに重ねて書かないでください。

```json
{
  "files": {
    "manifest": "seseragi.toml",
    "guide": "guide.md",
    "expectedOutput": "stdout.txt"
  },
  "workspace": {
    "active": "main.ssrg",
    "open": ["main.ssrg", "feature/helper.ssrg"],
    "expanded": ["feature"]
  }
}
```

現在の複数ファイルのサンプルは `project-flow-app`、`project-greeting`、
`seseragi-landing-page`、`web-starter` の4つです。

### sample.json と Discover への表示

- `sample.json` の形式は [`sample.schema.json`](./sample.schema.json) で決まっています。
- 言語の基礎を順に学ぶ教材は [`../tour/`](../tour/) が担当します。
- Discover に出すサンプル（レシピやショーケース）は、[`discover-groups.json`](./discover-groups.json) のどれか1つのグループに置きます。
- Tour を作る元になった `lesson` 種別のサンプルは、Discover には表示しません。

### HTML サンプルの CSS クラス

`outputMode: "html"` のサンプルでは、Preview が用意するユーティリティ CSS と、ソース中の `class` が一致するかを `samples:check` が確かめます。

- 固定のクラスは、文字列か `cx [...]` に直接書きます。
- 式で組み立てるユーティリティクラスは `preview.dynamicUtilities` に宣言します。
- 見た目を持たない意味付けだけのクラスは `preview.customClasses` に宣言します。CSS は付かないので、見た目を変えたいときは `html.Style` を使います。

## Web UI サンプルの分類

`outputMode: "html"` のサンプルは、「どの段階の人が、どの書き方を、何のために見るサンプルか」を `sample.json` に書きます。

| 項目 | 値 | 意味 |
| --- | --- | --- |
| `experience` | `minimal` / `guided` / `showcase` | 入門・解説付き・完成例 |
| `architecture` | `static` / `dom-app` / `signal-run` / `signal-mount` / `multi-module` | 画面の動かし方 |
| `focus` | `component` / `state` / `form` / `event` / `composition` / `project` | 主に見せたいもの |
| `difficulty` | — | ソースを読むのに必要な言語知識 |
| `prerequisites` | サンプル ID | 先に見ておくとよいサンプル |
| `featured` | — | 今の Web UI 機能を代表する、最初に見るべきサンプルにだけ付ける |

テキストを出力するサンプルには、`experience`・`architecture`・`focus` を書きません。
HTML サンプルではこの3つが必須で、生成スクリプトは各役割のサンプルが1つ以上あることを確かめます。

現在の HTML サンプル:

| サンプル | experience | architecture | focus | 見どころ |
| --- | --- | --- | --- | --- |
| `html-components` | minimal | static | component | 状態なしで props・子要素・サーバー側描画を確認する |
| `interactive-app` | minimal | dom-app | state | 純粋な更新関数を簡易 API に渡す |
| `signal-run-route` | minimal | signal-run | state | 同じアプリを Signal・クエリ・`run` で明示的に書く |
| `feature-composition` | guided | signal-run | composition | 複数の Signal と、実行範囲の指定を組み合わせる |
| `form-todo` | showcase | signal-run | form | フォーム・入力チェック・複数のイベントをまとめる |
| `project-flow-app` | showcase | multi-module | project | 機能ごとのモジュール分割を Explorer で追う |
| `seseragi-landing-page` | showcase | multi-module | composition | 完成したページのセクション・状態・スタイルをモジュール単位で追う |

`signal-state` は DOM を使わない Signal の基礎なので、この分類には入れません。
上の7つはそれぞれ役割が違うので、似ていても重複として削除しないでください。
各サンプルの `guide.md` の冒頭には「このサンプルを選ぶ理由」を書き、
入門 → 解説付き → 1ファイルの完成例 → 複数モジュール、の順に次に見るサンプルを示します。

## ショーケースの品質基準

`kind: "showcase"` で `outputMode: "html"` のサンプルには、`showcase-review.json` を置きます。
形式は [`showcase-review.schema.json`](./showcase-review.schema.json)、
基準は [`docs/SHOWCASE_QUALITY.md`](../../docs/SHOWCASE_QUALITY.md) に従います。
最初の画面、レイアウトのリズム、見た目の個性、操作、コード構成について、意図を記録します。

人がデスクトップ、iPhone（幅390px）、Android（幅360px）、初期状態以外の代表的な状態、Preview、モバイルのコード表示を確認し、
その記録を保存してから `approval.status` を `approved` にします。
CI が通っただけ、よくあるカードの並び、プレースホルダーだけ、という状態ではショーケースとして追加できません。
承認したら、スクリーンショットの PNG とその SHA-256 を `visual-baselines.review.json` に記録します。

## HTML サンプルのソースの書き方

HTML サンプルのソースは、利用者がそのままコピーして使うお手本です。読みやすさを優先してください。

基本の並び順は次のとおりです。

1. `import`
2. データの型（`type` / `struct`）
3. `update`
4. スタイルの補助関数
5. 画面の部品（意味のまとまりごとのコンポーネント）
6. `mount` / `main`

説明は長いコメントにせず、関数名・型名と `guide.md` で伝えます。

### クラスの書き方

1〜4個で80文字以内に収まるクラスは、文字列のまま書いて構いません。
5個以上になる場合や、行が80文字を超える場合は、`cx` 関数と名前付きの値に分けます。
`cx [...]` は1行に1クラスずつ書きます（フォーマッタをかけてもこの形が保たれます）。

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

- 1ファイルのサンプルでは、`cx` を同じファイルに定義します。
- 複数ファイルのサンプルでは、`styles.ssrg` に `pub fn cx` を置き、使う側で明示的に import します。Explorer から実装をたどれるようにするためです。
- 状態によって変わるクラスは、大きなテンプレート文字列で組み立てず、意味のある名前を付けた値を `match` や `if` で選びます。

### `html.Style` を使う場面

ユーティリティ CSS にない値や、動的に変わる色・幅・トランジション、CSS カスタムプロパティには `html.style` を使います。
同じスタイルを2回以上使う場合や、1つの要素に複数のプロパティを指定する場合は、
`heroStyle` や `progressStyle mode` のように名前を付けた値・関数に切り出します。
URL を含む属性はスタイルで回避せず、`WebUrl` を使ってください（セキュリティのため）。

### コンポーネントの分け方

ページ全体を1つの大きな `view` に詰め込まず、ヒーロー、フォーム、サマリー、一覧、空のときの表示、ボタン群など、
画面上の意味のまとまりごとに分けます。
ただし、1行だけの薄いラッパーを増やす必要はありません。関数名と型を見れば props の意味が分かる単位で区切ります。

`samples:check` は、長すぎる `class` の直書き、1行に詰め込まれた大きな `cx`、定義元をたどれない `cx` を拒否します。

## 生成とチェック

```sh
cd apps/playground
bun run samples:generate
```

生成スクリプトはディレクトリを自動で見つけ、次をチェックします。

- `sample.json` の形式
- ID の重複
- ソースや期待出力ファイルの不足
- `prerequisites` の循環
- Discover のグループ設定（種別・参照先・重複なく1か所に置かれているか）

生成されるマニフェストにはソースのハッシュが入っていて、古くなっていれば `samples:check` が検出します。
ID の対応表などを手で編集する必要はありません。

`stdout.txt` は、実際の標準出力とバイト単位で一致させます。余分な末尾の改行は入れないでください。
説明はソースに書き込まず、`guide.md` に書きます。
