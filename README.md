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

# formatter
cargo run -p seseragi-cli -- format --check \
  examples/spec/artifacts/schema-1/rock-paper-scissors-cli/main.ssrg
```

`run`はsingle fileだけでなく、`seseragi.json`を持つlocal packageも受け取れます。
生成TypeScriptの実行にはBunを使います。

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
| `extensions/seseragi-spec-preview/` | TextMate grammarとnative LSPを接続するVS Code extension |

`runtime/ts`と`apps/playground`のTypeScriptは旧compilerではありません。前者は
Rust backendが生成コードへ接続するruntime、後者は同じRust driverをWASM経由で
呼ぶUIです。compiler実装をroot `src/`へ追加しないでください。

## Development

```sh
# CI-equivalent workspace gate
bun run check

# Rustとactive TypeScript sourcesをformat
bun run format

# native workspaceとPlayground bundleをbuild
bun run build
```

詳しい現在地と実装方針は
[STATUS](./docs/STATUS.md)、[ROADMAP](./docs/ROADMAP.md)、
[IMPLEMENTATION](./docs/IMPLEMENTATION.md)を参照してください。

## License

Apache-2.0
