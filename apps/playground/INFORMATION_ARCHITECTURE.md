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
- Showcaseは公開前に#245で定めるShowcase品質基準を満たす。

更新時はsampleの`featured`と`discover-groups.json`だけを変更し、manifestをgeneratorで更新して
`bun run check:playground`を実行します。

## Desktop / mobileの対応

| 意味 | Desktop | Mobile |
| --- | --- | --- |
| Playground | brandとeditor workspace | brandとeditor workspace |
| Tour | headerの`Tour` link | overflow menuの`Tour` link |
| Discover | headerの`Discover` button | toolbarの`Discover` button |

compact layoutで配置を変えても名称、遷移先、catalogの内容は変えません。Discover dialogは開いた直後から
Recipe / Showcaseを表示し、Tourへの中継tabを挟みません。

## 追加時の判断

新しい公開surfaceが必修の前提概念ならTourへ、一つの目的を解く例ならRecipeへ、完成形を見せる統合例なら
Showcaseへ追加します。APIを調べる入口はReference、任意の法則や設計背景はDeep Diveです。

Showcase追加作業はこのroutingを前提にし、情報設計を作り直さず内容と品質へ集中します。
