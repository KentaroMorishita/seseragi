# A Tour of Seseragi curriculum

`curriculum.json` は、現行compilerで実行できるsurfaceだけを使う14段階のcanonical Tour設計です。
Tour UIとlesson sourceを作る後続Issueは、lesson ID、順序、初出topic、前提、出力modeをこのartifactから読みます。

## 教材の境界

- TourはHello worldから小さなWeb appまでを一方向に積み上げる唯一のLearn導線です。
- 一lessonの中心概念は`focus`の1〜2個です。`introduces`はそのlessonで初めて説明する語彙です。
- Recipeは既習概念を目的別に組み合わせる例、Showcaseは複数surfaceを統合した完成例です。
- `examples/samples/` は現在実行できるsourceの正本です。`seedSamples`は教材作成時の実装根拠であり、sourceをそのまま複製する指示ではありません。
- `examples/spec/lessons/` は完成仕様のdesign curriculumです。現行stdlibにないsurfaceをTourへ持ち込む根拠には使いません。

## Lesson artifact

実装済みlessonの正本は`lessons/<lesson-id>/`です。各directoryは次のfileを持ちます。

- `lesson.json`: challenge、interactive性、教材fileの対応を宣言します。
- `main.ssrg`: 初期表示され、CLIとbrowserの両方でcompile・実行するsourceです。
- `guide.md`: sourceだけでは説明しない背景と構文の読み方です。
- `stdout.txt`: browser-interactiveでないlessonの期待出力です。

metadataの契約は`lesson.schema.json`、Playgroundから読む生成manifestは
`apps/playground/src/generated/tour-manifest.ts`です。manifestを直接編集せず、
`apps/playground`で`bun run tour:generate`を実行します。段階的な教材作成中は
未実装lessonだけが`seedSamples`の既存sourceへfallbackします。

## 14 lesson

| # | Lesson | 中心概念 | 初出surface | 担当 |
|---:|---|---|---|---:|
| 1 | 最小のmainと文字列出力 | program entry / text output | `main`, `effect fn`, String, `println` | #121 |
| 2 | 値・binding・型注釈 | immutable binding / type annotation | `let`, type annotation, template | #121 |
| 3 | 関数を定義する | function definition / return type | `fn`, parameter, pure function | #121 |
| 4 | 関数を呼び出す | application / curried arguments | application, currying, partial application | #121 |
| 5 | `$`と`\|>`で値を渡す | low-precedence application / pipeline | `$`, `\|>` | #121 |
| 6 | RecordとStructを組み立てる | named fields / immutable update | Record, Struct, field access, spread | #122 |
| 7 | データ型をPattern matchする | ADT / pattern matching | constructor, `match`, pattern binding | #122 |
| 8 | Collectionを変換する | finite collections / transformation | Array, List, Range, `map`, `filter` | #122 |
| 9 | 値がない場合と失敗を表す | optional value / typed failure value | Maybe, Either, Left, Right | #122 |
| 10 | Effectをdoで合成する | deferred effect / sequential composition | Effect, `do`, bind, `with`, `fails` | #123 |
| 11 | Genericな契約をTraitで表す | parametric abstraction / type class evidence | generic, trait, instance, impl, operator | #123 |
| 12 | Signalで時間変化する状態を扱う | time-varying value / atomic update | Signal, MutableSignal, Applicative, transaction | #123 |
| 13 | Function componentでWeb UIを作る | pure Html / function component | Html, Style, Preview | #123 |
| 14 | Typed Actionで小さなアプリを動かす | pure reducer / effectful action | typed Action, `dom.app`, `Task<Unit>` action | #123 |

各lessonは直前lessonだけを直接の`prerequisites`に持ちます。したがって順序自体がprerequisite graphの唯一のcanonical pathです。
topicは一つの`introduces`にだけ置き、後続lessonは説明済みとして使用します。

## 既存sampleの監査方針

`sampleAudit`は25 sampleを重複なく分類しています。
`currentPathDuplicates`は未解決の重複を表し、現在は空です。通常Playgroundの
[`discover-groups.json`](../samples/discover-groups.json)はRecipe / Showcaseを目的別groupへ
一度だけ配置し、Tour seed-only sampleは表示対象にしません。

- `tour-seed-only`: stable sourceは実装根拠として保持し、Learn上の役割はTourへ移します。
- `tour-seed-and-recipe`: 一部をTourへ借り、目的別の完成例はDiscoverのRecipeに残します。
- `tour-seed-and-showcase`: 縮小版をTourへ借り、統合例はDiscoverのShowcaseに残します。
- `discover-recipe`: Tourを水増しせず、既習概念を試すRecipeへ移します。
- `discover-showcase`: 統合完成例としてShowcaseに残します。

通常PlaygroundではLearnがTourへの単一導線、DiscoverがRecipe / Showcaseの検索・分類surfaceです。
stable slugとsource fileは役割の再分類後も維持します。

## 現行Tourへ入れない完成仕様surface

`excludedDesignSurfaces`は、design curriculumには存在するものの現行Tourへ公開しないsurfaceを明示します。
concurrency、stream、resource scope、transformer、bytes、decimal、regex、timezone、filesystem、JSON codec、temporal effect、BigIntを教材都合で捏造しません。
pure Htmlとinteractive DOMは現行実装があるためlesson 13〜14へ含めます。再帰、newtype、alias、Monoidなど実装済みでも14段階の中心線から外れるsurfaceはRecipeまたはReferenceへ送ります。
