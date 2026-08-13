# Playground information architecture

この文書は、Playgroundで公開する学習・探索surfaceの役割と導線の正本です。

## 利用者向けの三つの入口

| 入口 | 利用者の目的 | 内容と順序 |
| --- | --- | --- |
| Playground | codeを自由に編集して実行する | 現在のworkspace。sampleを読み込んだ後も編集できる |
| Tour | Seseragiを最初から順に学ぶ | `examples/tour/curriculum.json`が定める唯一の順序付き経路 |
| Discover | やりたいことに近い例を探す | Recipe / Showcaseだけをgroup、検索、filterから順不同で選ぶ |

`Learn`は独立したnavigation、tab、catalog名にしません。学習を案内するときは、内容と進捗を二重化せず、
明示的にTourへ導きます。

## Playgroundのstart state

PlaygroundはStarter-firstです。初回表示は`hello-world`のcanonical sampleをそのまま読み込み、初期化用の
source copyを別に持ちません。利用者はいつでも次を明示的に選べます。

- Blank workspace: 空の`main.ssrg`を持つscratch workspace。sample metadataやGuideを持たない。
- Hello world: canonical starter sample。Resetするとmanifest由来のsourceへ戻る。
- Discover: Recipe / Showcaseを選び、そのcanonical workspaceを読み込む。
- Tour: 順序付き教材pageへ移動する。

Resetは現在のBlankまたはsampleをcanonical stateへ戻す操作、New blankは現在のoriginに関係なく新しい空の
workspaceを作る操作です。どちらもdirty fileを黙って破棄しません。local persistenceから復元できた場合は、
Starter-firstの初期化より復元workspaceを優先します。

## 内容のrouting

- Lesson: 後続概念の前提になる一概念を段階的に教える。Tourだけに置く。
- Recipe: 既習概念を組み合わせ、一つの実用目的を達成する。Discoverへ一度だけ置く。
- Showcase: 複数の公開surfaceを統合した完成例。Discoverへ一度だけ置く。
- Minimal / Guided: Web UI例の説明量を表すfacet。内容種別やnavigationにはしない。
- Architecture / Focus: Discoverで比較しやすくするcard metadata。routingを増やさない。

Recipe / Showcaseは`discover-groups.json`の一つのgroupにだけ所属させます。LessonをDiscoverへ入れず、
同じsampleを別名のcardや固定HTMLとして複製しません。

## Featured

FeaturedはDiscoverを開いた利用者へ最初に薦めるRecipe / Showcaseです。

- 最大8件とする。
- 表示順はDiscover group順、その中のsample順とする。別の手動順序を持たない。
- 現行compiler、CLI、WASMで実行でき、source、Guide、metadataが一致しているものだけを選ぶ。
- Web UIはPreview、mobile layout、主要interactionを確認済みにする。
- Showcaseは公開前に#245と
  [`docs/SHOWCASE_QUALITY.md`](../../docs/SHOWCASE_QUALITY.md)で定める品質基準を満たす。

更新時はsampleの`featured`と`discover-groups.json`だけを変更し、manifestをgeneratorで更新して
`bun run check:playground`を実行します。

## Interaction architecture

操作は作用範囲を基準に、次の五層へ分けます。

| 層 | 責務 | 置き場所 |
| --- | --- | --- |
| Global / surface | 現在地、Playground / Tour切替、Tour進捗、Run、global overflow | global header |
| Workspace | sample / workspace選択、Discover、New blank、Reset、file / Explorer | workspace chrome |
| Editor command | active fileへのFormatなど | editor / file header |
| Pane control | Explorer、Curriculum、Outputの開閉・resize・表示切替 | 対象paneのheaderまたは境界 |
| Preference | editor表示とformat設定 | global overflowから開くshared Settings |

global headerへ新機能のbuttonを直接追加しません。まず作用範囲を判断し、workspace、editor、pane、Settingsの
いずれかへ置きます。global headerにはsurface全体に効く操作だけを残します。

### Surface switching

Playgroundでは`Playground ▾`、Tourでは`Tour ▾`を同じ位置・操作modelで表示します。どちらからも相互に
移動でき、desktopとmobileで常設の別linkや「戻る」buttonを増やしません。

Discoverはglobal surfaceではなく、Playground workspaceの選択方法です。workspace selectorを開くと
Blank / starter / Resetと同じ文脈でRecipe / Showcaseを検索できます。選択後も利用者が編集する対象は
Playground workspaceのままです。

### Pane ownership

- Explorerのopen controlはPlayground workspace chrome、close controlはExplorer headerに置く。
- Curriculumのopen / closeはTourのlocal navigationまたはpane境界に置く。
- FormatはPlayground / Tourともactive editorのheaderに置く。
- Input / Text / Preview / Clear / Full screenはOutput paneが所有し、global headerへ移さない。
- resize / collapseはlayout変更だけを行い、sourceや実行結果を暗黙に書き換えない。

### Shared Settings

Playground / Tourは同じ`EditorPreferences`とlocal persistence keyを使用します。`Show indentation
whitespace`とformatter line widthは同じSettings surfaceに置き、一方で変更した値を他方の読み込み時にも
反映します。desktopはcompact dialog、mobileはbottom sheetとして表示しますが、設定の意味・validation・
永続化modelは変えません。

## Desktop / mobileの対応

desktopはworkspace、editor、paneの横幅を活かしてlocal headerと境界へ操作を配置します。mobileはglobal
headerを`surface switcher / overflow / Run`の一行に保ち、その下へworkspace selectorとlocal pane navigationを
置きます。compact layoutで配置を変えても名称、作用範囲、遷移先、catalog内容は変えません。

## 追加時の判断

新しい公開surfaceが必修の前提概念ならTourへ、一つの目的を解く例ならRecipeへ、完成形を見せる統合例なら
Showcaseへ追加します。APIを調べる入口はReference、任意の法則や設計背景はDeep Diveです。

Showcase追加作業はこのroutingを前提にし、情報設計を作り直さず内容と品質へ集中します。
