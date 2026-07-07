# Compiler artifact contracts

このdirectoryは、compiler stage間で受け渡すdebug / conformance artifactのschema fixtureです。
言語の意味やpublic ABIではなく、複数実装laneを疎結合に検証するためのversioned test contractです。

`schema-1/basic/` は同じsourceに対する次の四artifactを固定します。

- `tokens.json`: triviaとEOFを含むlossless token列
- `cst.json`: token rangeを参照するlossless CST骨格
- `diagnostics.json`: frontend共通diagnostic envelope
- `interface.json`: module consumerが読む公開interface
- `surface-ast.json`: surface declaration skeleton。後続stageがある場合はstage chainの入口
- `resolved-ast.json`と`typed-hir.json`: Rust conformance runnerで個別に比較できるfrontend / semantics stage snapshot
- `core-ir.json`から`typescript-ir.json`: backend loweringとemitterを含む最小end-to-end snapshot
- `generated-module.json`と`main.ts`: runtime requirementを含むemitter結果
- `main.ts.map`: portable Seseragi URIとsourcesContentを持つsource map v3

`schema-1/recovery/`は式が欠けた編集中sourceを固定します。token列は入力を失わず、CSTはzero-widthの
`error-expr`とmissing expressionを持ち、diagnosticのprimaryも挿入位置のzero-width rangeです。Errorを持つ
moduleは公開interfaceへ不完全なsymbolを出しません。

`schema-1/effect-do/`は最小の`effect fn main`と空の`do {}` blockをfrontend artifactとして固定します。
TokenStreamとLosslessCstのproducer比較対象ですが、型検査やdo desugaringはまだ要求しません。

`schema-1/multiple-lets/`は複数top-level declarationのCST分割と、public declarationだけをinterfaceへ出す
最小contractです。

`token-schema-1/`はlexer lane専用のTokenStream fixtureです。CST、diagnostic、module interfaceを要求せず、
fixed operator、comment、literal、nested type argument表面構文、trivia、UTF-8 byte range、EOF、
effect / do表面構文、range operator、member access、lossless reconstructionだけを先に固定します。`>>`のような
operator runを型文脈で分割するかどうかはparser stageの責務で、lexer stageは入力textを失わず保存することを
優先します。

rangeはすべて0-based、end-exclusiveのUTF-8 byteです。tokenの`raw`をEOF以外すべて連結するとsourceと
byte単位で一致しなければなりません。CSTの`startToken` / `endToken`はtoken indexのhalf-open rangeです。

artifactのJSONは人が手で実装する入力ではありません。新compilerが生成し、conformance runnerが比較します。
schema fieldを変更するときはproducerとconsumerを同じ巨大changeへ混ぜず、schema fixture、producer、consumerの
順に移行します。schema majorが異なるartifactを黙って読み替えません。

`schema-1/*/typed-hir.json`は`resolved-ast.json`の後続stageとして単独で追加できます。TypedHir producerを
Rust conformance runnerへ接続するとき、同じfixtureに`core-ir.json`や`typescript-ir.json`を同時に固定する
必要はありません。backend loweringやTypeScript emitterまで固定するcaseだけが`core-ir.json`、
`typescript-ir.json`、`generated-module.json`を持ち、full lowering chainとして検査されます。

初期runner skeletonは次でartifact bundleを発見し、必須file、JSON envelope、参照snapshotを検査します。

```sh
bun run conformance:artifacts
```

machine-readableなreportだけが必要な場合は、Bunのcommand echoを抑えて次を使います。

```sh
bun run --silent conformance:artifacts --json
```

このrunnerはまだcompiler producerを起動しません。手書きcontractを読むconsumerを先に固定し、後からlexer、
parser、backend、host runnerのproducerを一つずつ接続します。

`interface.json`はsource bodyやbackend layoutを含めず、公開symbol、type scheme、operator、instance、dependencyだけを
持ちます。private bodyを必要とする最適化は同一compiler runのTypedHir / CoreIrを使い、interface cacheへ漏らしません。

`interface-schema-1/rich/`はmodule graph lane向けの独立bundleです。複数sourceを入力にし、main moduleから
dependency edge、公開newtype、公開custom operator、coherenceに必要なinstance headだけをinterfaceへ保存します。
frontend artifactと同じbundleへ押し込まず、module resolverがTokenStreamやCSTの内部shapeへ依存することを防ぎます。

`runtime-schema-1/core/abi.json`はTypeScript runtimeのfeature registryです。generated moduleは必要feature IDと
ABI majorだけを記録し、compiler内部のruntime helper pathを推測しません。import不要なInt表現もfeatureとして
登録し、EffectやADT helperと同じ互換性検査へ載せます。helper featureのimportはmodule / exportのstructured pairで、
TypeScriptIrはfeature IDとlocal bindingだけを保持します。

`stage-schema-1/effect-main/`は最初のEffect縦sliceです。parameterなし`effect fn`をimplicit Unit parameter、
closed Console requirement、ConsoleError failure、Unit successへ展開し、runtime featureからprintln importを
解決します。TypeScript backendはSeseragi EffectをPromiseやthrowへ勝手に変換せず、runtime Effect valueを返します。

`execution-schema-1/effect-main/`は、generated moduleが返すEffect valueをhost adapterがどう実行するかを
固定します。entry runnerは`main ()`を一度呼び、root resource scopeでEffectを実行し、required environmentへ
Console serviceを提供します。required environmentはclosedですが、actual host environmentは追加serviceを持てます。
成功時はUnit valueとexit code 0、Console traceとstdout snapshotを比較します。
