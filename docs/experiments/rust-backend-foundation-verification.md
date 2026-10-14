# Rust backend foundationの検証記録

2026-10-14、base `8d42c0eac8ce6d7525c6934563c636551bf95a0f`、
branch `codex/rust-backend-foundation`で検証した。設計は
[architecture/rust-backend.md](../architecture/rust-backend.md)を参照する。

## 完了した検証

- Rust 1.97.1 / Bun 1.3.11でbackend、shared driver、loweringのscoped Rust laneが成功。
- Backend runtimeの4 test：100,000段のADT / captureの256 KiB stackでの破棄、
  1 MB文字列の浅い共有、dictionary cacheの明示的解放、TLS teardown。
- Native executionとanalysisの8 test：6 source fixtureの1万回反復、parameter-dependent
  dictionaryのnegative control、完全適用 / 部分適用 / alias / 返却関数、binding scope、
  hidden evidence、unused Arrayを返却closureが保持しないこと、target診断、Int defectと評価順序。
- CLIの2 test：6 fixtureのnative / TS stdout一致、native PATH空での実行、
  emit-onlyがrustcを起動しないこと、unsupported sourceでの出力抑止、入力上書きの拒否。
- `bun run check`のRust workspaceは129個のmacOS test executableを実行し、全て成功。
- 全conformance、native samples、production artifact 6 fixture、development/releaseのWeb browser
  interaction、Playground build、extension package verificationが成功。
- `git diff --check`とREADME local link checkが成功。

runtime modelの深さ検証を、Seseragiソースのdeep recursion / TCOの証明としては扱わない。
メモリcounterは管理ノード数でありRSSではない。nativeの速度比較はまだしていない。

full gate後に追加したInt負号、explicit Unit entry、unused captureの検証は
`target/rust-backend-foundation/backend-final.log`のbackend / native CLI testsで再確認した。
shared driverとloweringのsourceはfull gate実行時から変更していない。

## Full gateの未完了境界

compiler / shared driver / CLI基盤を変更したため`bun run check`を実行した。結果はexit 1、
以下の5 stepが失敗した。

1. `scripts/check-collection.test.ts`
2. `runtime/ts/tests`のunicode artifact subprocess output検証
3. `scripts/sample-cli-process.test.ts`
4. committed Playground WASMのbyte比較
5. release / local dogfood / local Web E2E helperの一括Bun test

1 / 2 / 3 / 5では、Bun test中の子プロセスからstdout / stderrが空になるassertion failureを
観測した。compilerを読み込まない、`bun -e 'console.log(...)'`を子にする最小probeでも再現した。
同じBun childをtest runner外から起動すると期待どおり両方の出力を捕捉できた。
この差は新backendのsource semanticsの証明や成功扱いに読み替えない。該当harness、runtime、
release helperのsourceは変更していない。

4はmacOSで再生成したWASMとcommitted artifactの相違。正規生成hostはLinux x64であり、
macOS生成binaryは`target/rust-backend-foundation/macos-wasm/`へ保存して、committed fileを元に戻した。
この境界は下記Linux CIで生成した正式artifactへ置換し、PRのfreshness gateで再確認する。

Full gateをpassとは記録しない。main merge / canonical release / local dogfoodは行っていない。
ローカルlogは`target/rust-backend-foundation/{rust-lane,native-cli,capture,full-gate}.log`に保存した。

## CI

- [Rust backend Linux CI](https://github.com/KentaroMorishita/seseragi/actions/runs/38499304447)が
  `e91be6c1e6860e8de041a1216ee87f512c61a48f` / 0.61.30で成功。frontend boundary、
  runtime 4 test、native 8 test、CLI 2 testを確認した。
- [canonical Linux WASM生成](https://github.com/KentaroMorishita/seseragi/actions/runs/38499358281)の
  `convert (ubuntu-24.04)`が成功。同じsource SHAからrepo scriptで生成した
  `canonical-playground-wasm-e91be6c1e6860e8de041a1216ee87f512c61a48f`を取り込んだ。
  変換器のmemory-only boundary、browser converter / workspace tests、TypeScript check、
  browser binding E2Eも同jobで成功。これはプログラムのRust→WASM生成ではなく、既存compiler adapterの確認。
- version bump直後のcommitではCargo.lockの同期が漏れて`--locked` CIが失敗した。
  `cargo update --workspace --offline`でworkspace crateのみを同期し、上記success SHAで修正を確認した。
- [draft PR #808](https://github.com/KentaroMorishita/seseragi/pull/808)に設計・実験・実装を保存している。
  main merge / tag / releaseは実行していない。


`.github/workflows/rust-backend.yml`でLinuxのshared frontend、runtime / native backend、
CLIのTS比較を固定する。既存Portable DTS workflowの`refresh_artifact`でcanonical Linux WASMを
生成し、source SHAとartifactの対応を確認してから取り込む。CI結果はPR / runで別途確認する。
