# Seseragi roadmap

このroadmapは、言語能力の完成順と、その能力を人間が利用・検証するproduct surfaceを別軸として扱います。
CLI、LSP、playgroundは完成後に載せる別製品ではありません。各Phaseのcompiler entrypoint、structured
diagnostics、runtime boundaryを実際の利用経路から検証する縦sliceです。

## 言語能力のPhase

| Phase | 利用者が書けるprogram | 主な一般機構 |
| ----- | --------------------- | ------------ |
| 0 compiler基盤 | sourceを全artifactへ変換し、同じ診断を再現できる | source / CST / AST / resolver / TypedHir / CoreIr / backend / diagnostics |
| 1 single-file言語 | 一つの`.ssrg`でADT、match、typed Effect、Console / Stdinを組み合わせたprogram | single-module driver、cold Effect、host runtime、user-facing file execution |
| 2 module / package | Phase 1 programを複数moduleとpackageへ分割できる | module identity、manifest、filesystem discovery、link、instance closure、package entry |
| 3 generic抽象化 | user定義generic ADT、HKT、trait、instance、Functor / Applicative / Monadを組み合わせられる | kind inference、constraint solving、coherence、dictionary evidence、generic lowering |
| 4 resource / concurrency | resource lifetimeと並行taskをtyped Effect内で安全に構成できる | scope、acquire / release、cancellation、structured concurrency、Stream / Signal runtime |
| 5 stdlib / interop | 実用的なapplicationを標準APIとTypeScript / JavaScript境界へ接続できる | collection、filesystem、HTTP、codec、foreign ABI、DOM / SSR |

Phase番号は言語能力の依存順です。product surfaceの実装をPhase 5の後まで待つ意味ではありません。

## Phaseごとのproduct surface gate

| Surface | Phase 0 | Phase 1 | Phase 2以降の伸ばし方 |
| ------- | ------- | ------- | --------------------- |
| CLI | structured diagnosticsを表示するthin adapter | **`seseragi run path/to/app.ssrg`**でsingle-fileをcompile / run | **local Package CLI完了:** manifest entryとlocal module graphを`seseragi run .`でcompile / run。dependency package対応を後続追加 |
| LSP | **LSP-0完了:** open documentを同じdriverでparse / resolve / typeし、source range付きdiagnosticsを返す | Phase 1構文のdocument diagnostics | module graph、cross-file definition / reference、package diagnostics |
| playground / WASM | **Playground-0完了:** single-file sourceを同じdriverでcompileし、実行結果またはdiagnosticsを表示する | **Playground-1実装済み:** mobile-first editor、専用highlight、任意Stdin、structured diagnosticsを新UIで提供 | module input、resource制限、stdlibのbrowser capability |
| formatter | lossless CSTを共有し、parse recoveryを壊さない | Phase 1構文をround-trip | 各Phaseの新構文をgrammarと同じcommitで追加 |
| conformance | stage schemaとpositive / negative fixture | Phase 1累積programとactual execution | 小さい機能fixture＋過去能力を残した累積goal program |

CLI / LSP / WASM adapterはparser、resolver、type checker、loweringを所有しません。`seseragi-driver`のpublic
entrypointとstructured diagnosticsを使います。process executionは`seseragi-runtime`のhost boundaryを使い、
language semanticsをtarget adapterへ委譲しません。

## 現在のgate

- Phase 0: Rust compiler pipelineとconformance artifactの最小経路は接続済み。
- Phase 1: 型付きじゃんけんprogramのcompiler / runtime fixtureに加え、fixture metadataなしの
  `seseragi run path/to/app.ssrg`を接続済み。
- Phase 2: linked compileとproject executionは進行中です。strict manifest、canonical local discovery、shared project driver、
  multi-module runtimeを接続し、分割じゃんけんpackageを`seseragi run .`で実行できます。registry / alias / path dependencyの
  typed manifest contractに加え、canonical path dependencyを再帰発見するlocal package graphを接続済みです。依存先manifestの
  name / language照合、cycle、同名同versionの別source混入をproject layerで拒否します。bare importはimporterの宣言済み
  dependency keyを最長prefixで選び、exact package identityの公開export subpathへ解決します。複数packageのcanonical source
  discoveryまで接続し、path dependencyをまたぐ閉じた`ModuleIdentity` graphを構築できます。cross-package driver compileと
  source root全体のcollision auditは未完了なので、現在のPackage CLIはcompatibleなlanguage rangeを持つrelative / `self/`
  importだけの一package executionを対象にします。
- LSP-0: `seseragi-lsp`がstdio JSON-RPC、position encoding negotiation、open / full-change / closeの
  diagnosticsをshared driver上で提供済み。hover、completion、module graphはこのgateに含めません。
- Playground-0: `seseragi-wasm`がshared driverとentry contractをbrowserへ公開し、playgroundのRunは旧TS
  parserではなくWASM compile、generated TypeScript、browser Console / Stdin hostを通ります。lesson 01と
  Phase 1累積じゃんけんの実行をintegration testで固定済みです。
- Playground-1: `apps/playground`へ旧UIと独立したCodeMirror 6のmobile-first surfaceを実装済みです。
  diagnosticsのUTF-8 range変換、専用syntax highlight、任意Stdin、responsive panelを小さいmoduleへ分離し、
  Vercelはversioned WASM deployment artifactをViteでbundleするためbuild hostのRust toolchainへ依存しません。
- formatter-0: `seseragi-formatter`がlossless token / CSTを入力にし、shared driver経由の
  `seseragi format [--check] path.ssrg`を提供済みです。Phase 1累積programのidempotent round-tripと
  compile artifact不変、CRLFからLF、2-space indent、trailing whitespace、末尾newlineを固定しました。
  parse recovery treeはtokenを変更せず、CLIはshared structured diagnosticsを返します。resolved fixityを
  必要とするoperator spacing / line wrappingとrange / stdin formatは、独自precedenceを作らず後続gateで接続します。

## 完了判定

各言語Phaseは、小さいpositive / negative fixtureだけでなく、過去Phaseの能力を残した累積goal programを
compile、diagnostics、CoreIr、TypeScriptIr、generated code、該当runtime executionまで通して完了とします。
標準型の名前やcompiler-private helperだけをhardcodeしてuser-defined programへ同等の表現力が渡らない経路は、
完了条件に含めません。

single-fileとbrowser runtimeで実行可能な言語機能は、compiler縦sliceが安定した時点でPlaygroundへ学習・実行用sampleを
追加します。module / package機能はsingle-file sourceへ擬似的に埋め込まず、Playgroundがshared driverのmodule inputを
受け取れるgateで累積sampleへ接続します。
