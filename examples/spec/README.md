# Seseragi specification examples

このdirectoryは刷新仕様に対する学習教材とconformance targetです。現行compilerとの互換exampleでは
ありません。読みやすさと機械的網羅性を一fileへ同居させず、二層に分けます。

仕様、fixture、実装の現在の境界は [`docs/STATUS.md`](../../docs/STATUS.md) を参照してください。

- [`lessons/`](./lessons/README.md): 人が順番に学ぶ実行可能program。
- [`fixtures/`](./fixtures/README.md): compiler / formatter / LSP向けの最小caseと失敗case。
- [`COVERAGE.md`](./COVERAGE.md): 仕様機能からlesson / fixtureへの対応表。

各sourceは次を満たします。

- `docs/spec/` の正本に定義された構文とAPIだけを使う。
- host service requirementとfailure型をmainの型へ明示する。
- 期待する出力または値をfile内に記録する。
- parse、type check、format、実行、LSP、highlight、playgroundで同じsourceを使う。
- playgroundへ載せる場合もsourceを複製せず、このfileから生成または直接読み込む。

lessonを追加するときは、一般的なcodeを書くために毎回自前実装が必要になった処理を記録します。
複数のexampleで繰り返す純粋操作、Effect operation、decoder、resource処理は標準ライブラリ候補です。
一例だけに固有なdomain helperはstdlibへ昇格させません。

`// Expected stdout: ./name.stdout` を持つexampleは、隣接するsnapshot fileとstdoutをbyte単位で
比較します。snapshotはUTF-8・LFで、末尾newlineも期待値に含みます。inlineの `// Expected stdout:`
以降へ出力を書く短いexampleも、各comment行から先頭の `// ` だけを除いたUTF-8・LF文字列として
同じように比較します。

学習順と各lessonの目的は [`lessons/README.md`](./lessons/README.md) を正本にします。番号は難易度順で、
新しいlessonを途中へ挿入する場合は参照とcoverage表を同時に更新します。
