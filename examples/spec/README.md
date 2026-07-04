# Seseragi executable design examples

このdirectoryは刷新仕様に対する実行可能な設計exampleです。現行compilerとの互換exampleでは
なく、新しいcompiler、formatter、language server、syntax highlighter、playgroundが共通して
扱うconformance targetです。

各exampleは次を満たします。

- `docs/spec/` の正本に定義された構文とAPIだけを使う。
- host service requirementとfailure型をmainの型へ明示する。
- 期待する出力または値をfile内に記録する。
- parse、type check、format、実行、LSP、highlight、playgroundで同じsourceを使う。
- playgroundへ載せる場合もsourceを複製せず、このfileから生成または直接読み込む。

exampleを追加するときは、一般的なcodeを書くために毎回自前実装が必要になった処理を記録します。
複数のexampleで繰り返す純粋操作、Effect operation、decoder、resource処理は標準ライブラリ候補です。
一例だけに固有なdomain helperはstdlibへ昇格させません。

`// Expected stdout: ./name.stdout` を持つexampleは、隣接するsnapshot fileとstdoutをbyte単位で
比較します。snapshotはUTF-8・LFで、末尾newlineも期待値に含みます。inlineの `// Expected stdout:`
以降へ出力を書く短いexampleも、各comment行から先頭の `// ` だけを除いたUTF-8・LF文字列として
同じように比較します。

## 一覧

- `01-fizzbuzz.ssrg`: tuple pattern match、template、range、`effect fn`、effectful `for`、`$`、
  Console出力、exact stdout snapshot。
- `02-word-count.ssrg`: multi-parameter lambda、text pipeline、generic map/reduce、Map upsert、
  deterministic iteration。
- `03-domain-types.ssrg`: newtype、constructor pattern、deriving、struct/newtype operator overload。
- `04-collections.ssrg`: Array module、generic map、stable sort、typed chunks、effectful iteration。
- `05-signals.ssrg`: MutableSignal、derived Signal、multi-signal transaction、`*signal` snapshot read。
