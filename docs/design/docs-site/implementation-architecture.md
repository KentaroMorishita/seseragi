# Docs Reboot 実装

公開catalogの正本は `apps/site/src/navigation/catalog.ssrg`。
Home/Docs/Examples/First Run/Releasesは `pages/entrance.ssrg`、
最初の章は三つのpage-owned article、文脈の章は `pages/composition.ssrg`、
処理と失敗の章は `pages/effects.ssrg`、
型とパターンの章は `pages/types.ssrg`、
Signalの章は `pages/signals.ssrg`、
APIの索引とmodule pageは `pages/api.ssrg` が持つ。
検索の本文・form・fallbackは `pages/search.ssrg` と `components/docs-search.ssrg` が持つ。
旧固定templateと旧symbol別記事catalogは退役した。

本文はSeseragiのtyped `Block` を記事ごとの順序で組む。
SSGのrender/batchは完全catalogを受け取り、最大64ページずつ生成する。
TypeScript hostはcanonical source/compiler metadataを渡し、内部linkを検証し、
assetsとmanifestを発行する。別のMarkdown/TypeScript本文rendererは設けない。

APIは `scripts/reference.ts` がcompilerのpublic schemaから宣言とinstanceを読む。
`ModuleSignatures`は一度moduleを選び、その宣言を表示する。各APIの定型文章や
型の読み方の量産は行わず、主要moduleのusageと難所を読む章に編集を集中する。

検索索引は公開HTMLの本文と同じcompiler metadataからlocale別に生成し、
module pageに実在する宣言anchorを検証する。`client/search.ts` は入力と候補の更新だけを担う。
検索ページだけがこのscriptと索引を読む。ネットワークAPIや外部検索は使わず、
`connect-src 'none'` のまま自己配信するmoduleを読む。no-JS/読込失敗時も章とAPI索引へ進める。

実装・型の正本は `docs/spec` とcompiler。旧route/anchor/ページ数の保全は不要。
独立アプリのPlaygroundは変更しない。品質検証は `scripts/check.ts` に集約する。
