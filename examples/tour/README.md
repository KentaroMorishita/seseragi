# A Tour of Seseragi curriculum

固定 lesson 数を廃止する再設計の category、lesson 順、prerequisite graph は
[`curriculum-map.md`](./curriculum-map.md)を正本とします。以下は移行完了まで動作する
現行 schema 2 artifact の説明です。

必修のTourと任意の理論・設計背景を扱う教材層の境界は
[`deep-dive-boundary.md`](./deep-dive-boundary.md)を正本とします。
標準Functor / Applicative / Monad instanceを具体型のどこで扱うかは
[`standard-instance-coverage.md`](./standard-instance-coverage.md)に記録します。

`curriculum.json` は、現行compilerで実行できるsurfaceだけを使うcanonical Tour設計です。
category → chapter → lessonをnested dataとして保持し、Tour UI、routing、progress、
前後移動は同じ配列から導出します。

## 教材の境界

- TourはHello worldから小さなWeb appまでを一方向に積み上げる唯一のLearn導線です。
- 一lessonの中心概念は`focus`の1〜2個です。`introduces`はそのlessonで初めて説明する語彙です。
- Recipeは既習概念を目的別に組み合わせる例、Showcaseは複数surfaceを統合した完成例です。
- `examples/samples/` は現在実行できるsourceの正本です。`seedSamples`は教材作成時の実装根拠であり、sourceをそのまま複製する指示ではありません。
- `examples/spec/lessons/` は完成仕様のdesign curriculumです。現行stdlibにないsurfaceをTourへ持ち込む根拠には使いません。

## Lesson artifact

実装済みlessonの正本は`lessons/<lesson-id>/`です。各directoryは次のfileを持ちます。

- `lesson.json`: 共通教材section、interactive性、教材fileの対応を宣言します。
- `main.ssrg`: 初期表示され、CLIとbrowserの両方でcompile・実行するsourceです。
- `stdout.txt`: browser-interactiveでないlessonの期待出力です。
- `exercise.ssrg` / `exercise.stdout.txt`: 一箇所を変更して確認する課題の初期sourceと期待結果です。
- `diagnostic.ssrg` / `diagnostic.txt`: よくある間違いとnative compilerの実出力snapshotです。

metadataの契約は`lesson.schema.json`、Playgroundから読む生成manifestは
`apps/playground/src/generated/tour-manifest.ts`です。manifestを直接編集せず、
`apps/playground`で`bun run tour:generate`を実行します。

### 共通lesson format

`formatVersion: 2`のlessonは、curriculum側のgoal・prerequisite・実行modeと
`lesson.json`の固有本文を組み合わせて次の必須sectionを作ります。

1. 今回できるようになること
2. 前提lesson
3. そのままRunできる完全なsource
4. expected outputまたはPreview
5. source line / rangeへ対応したwalkthrough
6. 新しく導入する構文・型・API
7. 一箇所だけ変更するexercise
8. native compiler出力を固定したdiagnostic example
9. 振り返り
10. 次lessonとの接続

`notes`だけがoptionalです。その他のsectionやartifact参照が欠けるとTour generatorが
失敗します。walkthroughのcode excerptは`main.ssrg`のline rangeからUIが導出するため、
説明用codeを二重管理しません。structured lessonは`guide.md`と同じ説明をsource commentへ
複製せず、section本文とcanonical sourceを分離します。未移行lessonの`guide.md` /
`challenge`は各delivery Issueで段階的にformat 2へ置き換えます。

exerciseの`reset` contractは`restore-lesson-source`です。課題や失敗例を開いた後も
Tour上部のResetでcanonical `main.ssrg`へ戻ります。diagnostic snapshotを更新するときは
repository rootで次を実行し、必ずnative compilerの出力から再生成します。

```sh
bun run tour:diagnostics:update
```

通常の`bun run test:samples:cli`はexerciseの実行結果とdiagnostic snapshotのfreshnessを
検証し、差分を自動更新しません。

## Data model

- category、chapter、lessonはそれぞれstable ID、表示順、title、summaryを持ちます。
- lessonはgoal、focus、introduced / required surface、複数prerequisiteを持ちます。
- lesson IDは表示順を含む必要がなく、件数や数値幅に上限はありません。
- `content`は`lessons/<stable-id>/lesson.json`を指し、source、section本文、stdin、
  expected output、exercise、diagnostic exampleを同じdescriptorから解決します。
- text output、static HTML Preview、interactive DOM Previewはcapabilityとoutput modeから
  導出します。
- local progressはstable lesson IDで保存し、schema 1のprogressをschema 2へ移行します。

## 自動coverage検証

`requiredTopics`は、このTourで必ず説明するtopicの独立したchecklistです。各topicは一つのlessonだけで初出し、
lesson側だけ、またはchecklist側だけを変更するとTour generatorが失敗します。generatorはさらに次を検証します。

差分のない対応表は[`coverage-report.md`](./coverage-report.md)へ生成します。checklistまたはlesson側だけが
変わった場合は、missing surfaceと余分なsurfaceをbuild errorへ表示します。

- category / chapter / lessonのstable IDと表示順が重複しない
- prerequisiteの参照先、cycle、canonical rootから到達できないlesson、canonical pathより後へのedgeを拒否する
- `requiredSurfaces`が以前のlessonで導入済みであることと、`focus`が中心概念1〜2個であることを検証する
- canonical content directoryとmanifestのlesson順が完全一致する
- non-interactive lessonがexpected outputを持ち、interactive flagとDOM capabilityが一致する
- format 2の必須section、exercise / diagnostic artifact、source line range、
  next lesson接続が一致する
- exerciseのformat・実行結果とdiagnosticのnative compiler snapshotが一致する
- sample auditが実metadataと一致し、各lessonにsample audit coverageと実在するseedがある
- `excludedDesignSurfaces`のtopicとmodule importがlesson source / guideへ混入しない
- #124で解消したsample path重複が再導入されない

compiler fixtureの網羅性は`examples/spec/COVERAGE.md`で別に管理します。Tourのcompile・実行成功だけでparser、
diagnostic、lowering、ABIのfixture coverageを代替しません。公開surface追加時のTour / Recipe / Showcase / Referenceへの
routingは`docs/spec/12-tooling.md`に記録しています。

## 既存sampleの監査方針

`sampleAudit`は通常Playgroundの全sampleを重複なく分類しています。
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
pure Htmlとinteractive DOMは現行実装があるためTourへ含めます。再帰、newtype、alias、
Monoidなど実装済みでもcanonical pathの中心線から外れるsurfaceはRecipeまたはReferenceへ送ります。
