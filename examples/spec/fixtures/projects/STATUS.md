# Project fixture status

このfileは `inventory.json` から生成します。directoryの存在だけを実装済みの根拠にせず、
`current` は通常product routeのtest evidenceを持つfixtureだけを表します。

## Current product-route fixtures (20)

| Fixture | Phase | Runner | Evidence |
| --- | --- | --- | --- |
| `cli-build-nested` | `run` | `cli-build`, `cli-run` | `crates/seseragi-cli/tests/build.rs` |
| `dom-hydration-mismatch` | `run` | `cli-build` | `apps/playground/tests/dom-lifecycle-browser.test.ts` |
| `dom-reactive-bindings` | `run` | `cli-build` | `apps/playground/tests/dom-lifecycle-browser.test.ts` |
| `dom-signal-lifecycle` | `run` | `cli-build` | `apps/playground/tests/dom-lifecycle-browser.test.ts` |
| `effect-resource-scope` | `run` | `cli-run`, `wasm-project` | `crates/seseragi-cli/tests/run.rs`<br>`apps/playground/tests/playground.integration.test.ts` |
| `effect-temporal-control` | `run` | `cli-run`, `wasm-project` | `crates/seseragi-cli/tests/run.rs`<br>`apps/playground/tests/playground.integration.test.ts` |
| `entry-rooted-runtime` | `run` | `cli-run`, `wasm-project` | `crates/seseragi-cli/tests/run.rs`<br>`apps/playground/tests/playground.integration.test.ts` |
| `imported-derived-json-codecs` | `run` | `cli-run` | `crates/seseragi-cli/tests/run.rs` |
| `logical-short-circuit` | `run` | `cli-run`, `wasm-project` | `crates/seseragi-cli/tests/run.rs`<br>`apps/playground/tests/playground.integration.test.ts` |
| `module-generic-nominal-identity` | `run` | `wasm-project` | `apps/playground/tests/playground.integration.test.ts` |
| `namespaced-reduce-rejection` | `diagnostic` | `cli-build`, `wasm-project` | `crates/seseragi-cli/tests/build.rs`<br>`apps/playground/tests/playground.integration.test.ts` |
| `package-path-dependency` | `run` | `project-loader` | `crates/seseragi-project/src/local_project/tests.rs`<br>`crates/seseragi-driver/src/local_project.rs` |
| `package-path-dependency-basic` | `run` | `cli-run`, `project-loader` | `crates/seseragi-cli/tests/run.rs`<br>`crates/seseragi-driver/src/local_project.rs` |
| `package-stale-lock` | `diagnostic` | `project-loader` | `crates/seseragi-project/src/lockfile/tests.rs` |
| `prelude-reduce-lambda` | `run` | `cli-run`, `wasm-project` | `crates/seseragi-cli/tests/run.rs`<br>`apps/playground/tests/playground.integration.test.ts` |
| `provider-http-client-e2e` | `run` | `cli-run` | `crates/seseragi-cli/tests/run.rs` |
| `provider-http-server-e2e` | `run` | `cli-run` | `crates/seseragi-cli/tests/run.rs` |
| `struct-field-generic-identity` | `run` | `wasm-project` | `apps/playground/tests/playground.integration.test.ts` |
| `std-parity-portable` | `run` | `cli-build`, `cli-run`, `lsp-project`, `wasm-project` | `crates/seseragi-cli/tests/build.rs`<br>`crates/seseragi-cli/tests/run.rs`<br>`crates/seseragi-lsp/tests/stdio.rs`<br>`apps/playground/tests/playground.integration.test.ts` |
| `std-parity-target` | `diagnostic` | `cli-run`, `wasm-project` | `crates/seseragi-cli/tests/run.rs`<br>`apps/playground/tests/playground.integration.test.ts` |

## Contract-only fixtures (37)

| Fixture | Phase | Runner | Evidence |
| --- | --- | --- | --- |
| `benchmark-discovery` | `tooling` | `planned-tooling` | - |
| `child-process-captured` | `run` | `planned-conformance` | - |
| `doc-tests` | `tooling` | `planned-tooling` | - |
| `dts-basic-conversion` | `convert` | `planned-converter` | - |
| `dts-callback-during-call` | `convert` | `planned-converter` | - |
| `dts-callback-missing-release` | `diagnostic` | `planned-converter` | - |
| `dts-declaration-merge` | `convert` | `planned-converter` | - |
| `dts-generated-name` | `convert` | `planned-converter` | - |
| `dts-namespace-runtime` | `convert` | `planned-converter` | - |
| `dts-unsupported-any` | `diagnostic` | `planned-converter` | - |
| `effect-stream-simultaneous-failure` | `run` | `planned-conformance` | - |
| `filesystem-temporary-cleanup` | `run` | `planned-conformance` | - |
| `foreign-pure-load` | `run` | `planned-conformance` | - |
| `foreign-task-load` | `run` | `planned-conformance` | - |
| `foreign-task-single-flight` | `run` | `planned-conformance` | - |
| `http-non-success-response` | `run` | `planned-conformance` | - |
| `http-stream-events` | `run` | `planned-conformance` | - |
| `imported-derived-show-debug` | `run` | `planned-conformance` | - |
| `modules-cycle` | `diagnostic` | `planned-conformance` | - |
| `modules-private-access` | `diagnostic` | `planned-conformance` | - |
| `modules-reexport-run` | `run` | `planned-conformance` | - |
| `package-invalid-manifest` | `diagnostic` | `planned-conformance` | - |
| `package-undeclared-dependency` | `diagnostic` | `planned-conformance` | - |
| `performance-profile-equivalence` | `run` | `planned-conformance` | - |
| `performance-release-shapes` | `compile` | `planned-conformance` | - |
| `performance-stack-safety` | `run` | `planned-conformance` | - |
| `process-shutdown-cancel` | `run` | `planned-conformance` | - |
| `process-shutdown-forward` | `run` | `planned-conformance` | - |
| `random-seed` | `run` | `planned-conformance` | - |
| `signal-transaction-lifetime` | `run` | `planned-conformance` | - |
| `source-map-rejection` | `run` | `planned-conformance` | - |
| `stdin-lines` | `run` | `planned-conformance` | - |
| `stream-cold-resource` | `run` | `planned-conformance` | - |
| `target-capabilities` | `tooling` | `planned-tooling` | - |
| `test-discovery` | `test` | `planned-tooling` | - |
| `typescript-abi-constrained` | `diagnostic` | `planned-tooling` | - |
| `typescript-abi-generic` | `tooling` | `planned-tooling` | - |

## Promotion rule

`contract-only` を `current` へ昇格する変更は、planned runnerを通常の CLI / LSP / project loader / WASM routeへ置き換え、
fixture directoryを直接参照するtest evidenceを同じ変更で追加します。inventory checkerはevidence fileの存在とfixture参照を検証します。
