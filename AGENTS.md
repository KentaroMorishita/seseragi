# Repository Guidelines

## Canonical implementation boundary

- `crates/`: current Rust compiler, driver, CLI, LSP, WASM adapter, runtime
  staging, and conformance runner.
- `runtime/ts/`: active TypeScript runtime used by generated programs.
- `apps/playground/`: active browser Playground using the Rust WASM driver.
- `examples/spec/`: canonical lessons, fixtures, and executable artifacts.
- `docs/spec/`: normative language specification.
- `extensions/seseragi-spec-preview/`: syntax-only VS Code support.

The former root TypeScript compiler and React/Monaco Playground were removed
after the Rust migration. Do not recreate compiler code under root `src/`, and
do not treat `runtime/ts` or `apps/playground` as legacy code.

## Build, test, and development commands

- `bun run build`: build the Rust workspace and production Playground bundle.
- `bun run check`: run formatting checks, Rust workspace tests, spec/example
  validation, conformance artifacts, WASM freshness checks, and Playground QA.
- `bun run format`: format Rust and active TypeScript/HTML sources.
- `cargo run -p seseragi-cli -- run <path>`: run a source file or package.
- `cargo run -p seseragi-cli -- format [--check] <path>`: format source.
- `bun run dev:playground`: run the local Playground.
- `bun run build:playground:wasm`: regenerate the committed WASM artifact
  after compiler, runtime contract, or WASM adapter changes.

## Coding conventions

- Rust uses `cargo fmt`; keep crate responsibilities narrow and reuse the
  shared driver, diagnostics, Typed HIR, lowering, and runtime ABI.
- Active TypeScript uses Biome: 2 spaces, LF, 80 columns, double quotes.
- Never hand-edit `apps/playground/src/wasm/pkg`; regenerate it through the
  repository script.
- Keep user-visible samples grounded in `examples/spec`. A language slice is
  complete only when parser/semantics/lowering/runtime behavior and diagnostics
  are verified at the appropriate boundaries.

## Testing and commits

- Add focused Rust tests and execution fixtures for compiler changes.
- Add Bun tests under `apps/playground/tests` for browser/UI behavior.
- Run `bun run check` before pushing.
- Use concise, imperative, scope-first commit subjects.
- Preserve unrelated worktree changes and never edit generated `dist/`
  artifacts by hand.

