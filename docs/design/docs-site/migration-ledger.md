# Docs Reboot 制作記録

入口は [#601](https://github.com/KentaroMorishita/seseragi/issues/601)、
編集の正本は [editorial-contract.md](editorial-contract.md)。
旧記事のKEEP/REWRITE/RETIRE台帳、旧URL/anchor/ページ数の保全条件は撤回済み。
新しい本文を必要性から選ぶ。旧記事との1対1対応や全件移植は要求しない。
以前のsnapshotはGit履歴に残る。

## 今回の公開構成

- Home: Hero、一つのfn/main例と実出力、短い紹介と章への入口。
- Docs: 目的と記法から同じ章・記事へ進む。
- 関数と記法: 検索キー、再利用する条件、検索から表示への合成。
- 型とパターン: record/aliasで設定をまとめ、ADT/matchで状態を分け、genericなpreviewにShowの制約を付ける。網羅性とtrait制約の負例も実診断を照合する。
- 文脈のある値: `<$>`で変換、`<*>`で引数を組み合わせ、`>>=`と`do`で依存する検索をつなぐ。
- 処理と失敗: Effectを値として持ち、doで順番に動かす。純粋なEitherの検証をfromEitherで持ち込み、recoverで回復する。
- 変わっていく値: Signalを<$>/<*>で組み合わせ、snapshotを読み、通知を記録して購読解除し、transactionで複数変更を一緒に公開する。process targetで実出力を照合。
- API Reference: モジュール単位の宣言集。identity/namespace/kindから安定したanchorを生成する。
  signature、制約、instance、対象targetはcompiler metadata由来。旧API別記事は生成しない。
  prelude/array/text/maybe/either/effect/console/signalには用途や使い分けと章への導線を添える。
- Examples、First Run、Releases: 実行可能なsource、正式リリース、実在する配布手順。
- 検索: 現在公開する日英の記事とcompiler由来のAPI宣言を探す。module名で同名関数を絞り、宣言anchorへ直接進む。no-JS/索引読込失敗でも章とmodule索引を辿れる。
- すべて日英の対。ページ数はcatalogとmetadataから決まり、旧件数を目標にしない。

## 実装の退役

使用しない1,570個の旧article/template/locale sourceと旧corpus向けcheckerを一括撤去。
`reference-article.ssrg`、`reader-article.ssrg`、`explanation.ssrg`、
`reference/editorial/model.ssrg`と旧symbol別catalogも含む。
新しい公開catalogが使うSeseragi sourceは49ファイル。
掲載しない971個の旧example/comparison/package filesも撤去し、掲載sourceだけを残した。
SSG、typed blocks、compiler metadata、syntax highlight、実行環境、静的配信handlerを再利用。
旧例の中から今の最初の章が使うsourceだけを公開チェックする。

## 品質ゲート

`bun run check:site` は次を確認する。

- Active siteのBiome/TypeScript。
- 現行CLIとcompiler standard-library metadataのfreshness。
- 実際に掲載する19例のstdoutと5負例の診断。各sourceを独立したfile entryで実行する。負例はCLIのJSON診断からcode/severityを照合し、新しい負例はmessageとlabelも一致させる。
- renderer protocol、内部リンク/fragment、静的配信、4GiB監視・cleanupの既存回帰。
- 本番SSGの全locale対・重複route/id・内部リンクと全artifact hash。
- 2回の完全buildの同一manifest。
- Browserで全公開route、全API signature/anchor、日英切替。
- 検索索引と公開HTML/metadataの一致、実在する宣言anchor、日本語・記号・module名による検索、40件制限・全件数、IME/keyboard/CSP・読込失敗fallback。
- 320/390/1280px、JavaScript有効/無効、mobile menu、章の通読導線。

通常のCI・保護ルールを使い、旧記事数・旧URLを守る検査を復活させない。
実行済み結果、SHA、PR、deployment URLと本番確認は#601とPRへ記録する。

## 未完了の範囲

作者の美学レビューと初見読者の理解は未確認。agentによる通読・実行・表示確認で
それを承認済みとはしない。本文の制作・公開は継続できる。
SignalのswitchMap/DOM、Effectの環境・resource・並列処理、struct/newtype・入れ子pattern、プロジェクト/interop等の新しい記事と、難解APIの追加usageは継続対象。
記事内編集・実行・型hoverは#768の追加体験。検索は現在の公開構成から生成済みで、旧corpusへ依存しない。
root Playgroundの配信切替は#631の別作業で、このDocs公開では実施しない。
