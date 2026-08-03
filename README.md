<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="./assets/brand/extension/logo-dark.jpg">
    <source media="(prefers-color-scheme: light)" srcset="./assets/brand/extension/logo-light.jpg">
    <img src="./assets/brand/extension/logo-light.jpg" alt="Seseragi" width="720">
  </picture>
</p>

# Seseragi

Seseragiは、独自の型・Effect semanticsを持ち、TypeScript / JavaScriptを
実行targetの一つとして生成するプログラミング言語です。言語の意味と構文の正本は
[Seseragi言語仕様](./docs/README.md)です。

現在のcompilerはRust実装です。旧TypeScript compilerは移行完了に伴って削除し、
parser、型検査、lowering、CLI、LSP、WASMはすべて `crates/` の同じdriver境界を
共有します。

## Quick start

必要なtoolchainはRust、Bun、PlaygroundのWASMを再生成する場合は
`wasm-pack`です。

```sh
# Rust CLIをbuild
cargo build -p seseragi-cli

# single-file programをcompileして実行
cargo run -p seseragi-cli -- run examples/samples/hello-world/main.ssrg

# 実行せず、再現可能なTypeScript成果物をdist/へ出力
cargo run -p seseragi-cli -- build examples/samples/hello-world/main.ssrg
bun run dist/entry.ts

# local packageのmodule graph全体を同じ成果物へ出力
cargo run -p seseragi-cli -- build \
  examples/spec/fixtures/projects/cli-build-nested

# formatter
cargo run -p seseragi-cli -- format --check \
  examples/spec/artifacts/schema-1/rock-paper-scissors-cli/main.ssrg
```

`run`はsingle fileだけでなく、`seseragi.json`を持つlocal packageも受け取れます。
生成TypeScriptの実行にはBunを使います。

`build`はsingle fileを既定の`dist/`、または`--out-dir`で指定したdirectoryへ
永続出力します。成果物には`main.ts`、`main.ts.map`、
`generated-module.json`、Bun用`entry.ts`、versioned TypeScript runtimeが
含まれます。同じ入力の再buildではdirectory全体を再生成しますが、
`.seseragi-build.json`を持たない既存の非空directoryは誤削除を避けるため
上書きしません。

local packageを渡した場合は、`run`と同じmanifest・module graph・entry解決を
使い、compilerが計画した`dist/packages/<name>/<version>/...`構成を保って
全moduleのTypeScript、source map、metadataを出力します。`entry.ts`から
実行するため、成果物directory内で`bun run entry.ts`を使えます。

## Release identity

CLIとLSPは同じtoolchain version、commit、build channelを公開します。

```sh
cargo run -p seseragi-cli -- --version
cargo run -p seseragi-lsp -- --version-json
bun run release:info
```

versionの正本、tag、artifact名、bump手順は[Release contract](./docs/RELEASE.md)を参照してください。

## Playground

```sh
# compiler / runtime contract変更時にWASMを再生成
bun run build:playground:wasm

# local development
bun run dev:playground
```

production相当のtest、typecheck、bundleは `bun run check:playground` で確認できます。
公開版は <https://seseragi.vercel.app/> です。

実装済みsurfaceだけで動く学習・発見用サンプルは `examples/samples/<slug>/` が正本です。
directoryへmetadata、source、guide、期待出力を追加するとmanifestへ自動検出されます。
`bun run test:samples:cli` は同じsourceをnative CLIでも検証します。

## Repository boundary

| Path | Role |
|---|---|
| `crates/` | 現行Rust compiler、driver、CLI、LSP、WASM、conformance |
| `runtime/ts/` | 生成コードが使う現行TypeScript runtimeとbrowser host |
| `apps/playground/` | 現行CodeMirror / WASM Playground |
| `examples/samples/` | 現行compilerで実行するsample catalog |
| `examples/spec/` | canonical lesson、fixture、execution artifact |
| `docs/spec/` | normative language specification |
| `extensions/seseragi/` | TextMate grammarと同梱native LSPを提供する正式なVS Code extension（ID: `seseragi-dev.seseragi`） |

`runtime/ts`と`apps/playground`のTypeScriptは旧compilerではありません。前者は
Rust backendが生成コードへ接続するruntime、後者は同じRust driverをWASM経由で
呼ぶUIです。compiler実装をroot `src/`へ追加しないでください。

## Development

```sh
# 変更範囲に応じた短い検証
bun run check:playground
bun run check:rust
bun run check:extension

# CI-equivalent repository-wide gate（統合時のみ）
bun run check

# Rustとactive TypeScript sourcesをformat
bun run format

# native workspaceとPlayground bundleをbuild
bun run build
```

詳しい現在地と実装方針は
[STATUS](./docs/STATUS.md)、[ROADMAP](./docs/ROADMAP.md)、
[IMPLEMENTATION](./docs/IMPLEMENTATION.md)を参照してください。

検証laneの選び方とfull gateの実行条件は
[Scoped checks](./docs/SCOPED_CHECKS.md)を参照してください。

## License

Apache-2.0
