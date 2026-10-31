# Repository scripts

Use this table to find the scripts for a task. Run commands from the repository
root. Only the probe harnesses are grouped in a subdirectory; the other groups
remain directly under `scripts/`.

| Group | Purpose | Main entry points |
| --- | --- | --- |
| Probes | Run generated TypeScript and program WASM in Bun and Node for Rust tests, and check archive transport. | [`probes/`](probes/); callers in [`crates/seseragi-cli/tests/`](../crates/seseragi-cli/tests/) and CI workflows supply the inputs. |
| Release | Check version metadata, package native tools, promote releases, and synchronize local tools. | `release-contract.ts`, `release-gate.ts`, `release-promotion.ts`, `native-release.ts`, `local-dogfood.ts` |
| Checks | Validate selected parts of the repository, fixtures, and production artifacts. | `check-scoped.sh` (`bun run check:rust`, `check:playground`, `check:extension`); `check.sh` (`bun run check`); `check-project-fixtures.ts` (`bun run fixtures:check`) |
| Generators | Update Playground lessons and runtime files, Unicode data, and provider bundles. | `generate-playground-*.ts`, `generate-unicode.ts`, `postgres-provider-bundle.ts`, `timezone-bundle.ts` |
| Others | Build and format the project, run platform checks, and support end-to-end tests. | `build.sh`, `format.sh`, `build-playground-wasm.sh`, `linux-native.sh`, `local-web-product-e2e.ts`, `run-macos-cargo-tests.ts`; shared test inputs live in `fixtures/`. |

See [`docs/SCOPED_CHECKS.md`](../docs/SCOPED_CHECKS.md) to choose checks for a change.
