# Repository Guidelines

## Project Structure & Module Organization
- `src/`: Core TypeScript sources — CLI (`cli/`), parser/formatter, LSP (`lsp/`), runtime, and shared types.
- `tests/`: Bun test suites (`*.test.ts`). Keep unit tests close to features they cover.
- `extensions/`: VS Code extension source (packaged by scripts).
- `playground/`: Demo app for local experimenting.
- `examples/`: Example `.ssrg` programs; use for docs and sanity checks.
- `dist/`: Build artifacts (generated; do not edit).

## Build, Test, and Development Commands
- `bun run dev`: Run the CLI in watch mode from `src/cli.ts`.
- `bun run build` or `./scripts/build.sh`: Clean and build LSP + extension artifacts into `dist/`.
- `bun run check` or `./scripts/check.sh`: Format+lint, type-check, and run tests.
- `bun run format` or `./scripts/format.sh`: Apply Biome formatting.
- `bun test` / `bun test --watch`: Run tests once / in watch mode.
- VS Code extension: `bun run vscode` to build/package and reinstall locally.

## Coding Style & Naming Conventions
- Language: TypeScript (ESNext), Bun runtime.
- Formatting via Biome: 2 spaces, LF line endings, 80 cols, double quotes, semicolons as needed, ES5 trailing commas. See `biome.json`.
- Naming: `camelCase` for vars/functions, `PascalCase` for types/classes, `SCREAMING_SNAKE_CASE` for constants.
- Files: Type-centric modules (e.g., `parser/lexer.ts`), tests mirror names (e.g., `lexer.test.ts`).

## Testing Guidelines
- Framework: Bun test runner.
- Location: `tests/` with `*.test.ts` naming.
- Coverage: Add tests for new features and bug fixes; prefer small, focused cases. Run `bun run check` before pushing.

## Commit & Pull Request Guidelines
- Commits: Imperative mood, concise scope-first subject (e.g., `parser: handle unicode escapes`). Group related changes.
- PRs: Include summary, rationale, and testing steps. Link issues (e.g., `Closes #123`). Add screenshots or CLI output for UX/LSP changes.
- Quality gate: CI-equivalent locally is `./scripts/check.sh`; PRs should pass and contain no formatting diffs.

## Security & Configuration Tips
- Do not commit secrets or machine-specific paths. The repo is `private: true` and uses local `@seseragi/runtime`.
- Generated code in `dist/` should be produced by scripts; never hand-edit.
- Requirements: Bun installed (`bun --version`), `code` CLI for extension packaging. If packaging fails, ensure `vsce` is available (script calls `bunx vsce`).

