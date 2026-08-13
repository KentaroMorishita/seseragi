# Project fixtures

一fileで表せないmodule graph、manifest、foreign binding、generated source、lockfileのcaseをdirectory単位で
置きます。各projectは独立した `seseragi.toml` を持ちます。

全project rootのroleは`inventory.json`を正本とし、`availability`、`phase`、`runners`、current fixtureの
`evidence`を機械可読に管理します。`project.expect.json`はplanned conformance runnerを含むcase固有の期待値であり、
存在だけでcurrent implementationとはみなしません。集約表は`bun run fixtures:generate`で`STATUS.md`へ生成します。

```json
{
  "schema": 1,
  "kind": "project",
  "phase": "run",
  "spec": ["6.10", "7.2"],
  "lock": "generate",
  "stdout": "expected.stdout"
}
```

- `phase`: `compile`、`diagnostic`、`run`、`test`、`convert`、`tooling`のいずれか。
- `availability`: `current`または`contract-only`。currentは通常product routeのtest evidenceがあるcaseだけに使います。
- `runners`: currentでは`cli-run`、`cli-build`、`lsp-project`、`project-loader`、`wasm-project`の組み合わせ、contract-onlyでは
  planned runnerを一つ指定します。
- `evidence`: currentへ昇格したfixtureをdirectoryから直接実行するtest source。専用synthetic resolverはevidenceにできません。
- `spec`: 根拠となる正本section。
- `lock`: `generate`または`fixture`。`generate`はrunnerがprojectをtemporary directoryへcopyし、offline resolverで
  lockfileを生成してから検証する。repository内へ生成物を書かない。`fixture`はproject内の`seseragi.lock`をそのまま
  使用し、更新しない。
- `stdout`: run / test fixtureのexact UTF-8 / LF snapshot。末尾newlineを含む。
- `stderr`: diagnostic detailやhost messageのexact UTF-8 / LF snapshot。末尾newlineを含む。
- `exitCode`: process-capable test targetのexpected code。省略時はsuccess phaseで0、diagnostic phaseでrunnerが
  command既定値を使う。
- `args`: fixture commandへmanifest optionの後で渡すstable CLI argument。shellで再解釈せず、array要素を
  一argumentとして渡す。
- `stdin`: run fixtureへtest adapterがそのまま渡すinput file。text / binary semanticsは対象APIの仕様に従う。
- `services`: deterministic test adapterへ渡すschema 1のJSON scenario。service operationを配列順に照合し、
  requestが一致しなければfixture failure、余ったresponseがあれば未消費fixture failureにする。network、real clock、
  machine filesystemへfallbackしない。
- `diagnostics`: diagnostic phaseで必須。single-file fixtureのdiagnostic objectへ、project rootからの
  `/` 区切りrelative `file` を加える。code、severity、UTF-8 byte range、anchor textをcheckerが検証する。
- `command`: 実行surface。diagnostic phaseでは省略時`compile`、converterなら`convert`、tool protocolなら
  `tooling`を指定します。tooling phaseでは正本が定義したcommandだけを明示でき、schema 1では`doc`を許します。
- `artifacts`: convert / tooling phaseの生成物。`output`はtemporary project内の生成先、`snapshot`はrepository内の
  exact expected textです。同じoutputを二度宣言できず、snapshotはUTF-8 / LF / final newlineを持つ。
- `shapes`: release compileの内部shape assertion。`symbol`は`module/path::declaration`、`require`は14.12で
  定義したpredicateの重複しないarrayです。`args`に`--profile`, `release`が必要で、IRのserialization自体は
  fixture contractにしません。
- `differentialProfiles`: run / test fixtureを指定profileごとに独立runtimeで実行し、14.12の観測結果を比較する
  canonical arrayです。schema 1では`["development", "release"]`だけを許し、`--profile`と併用しません。

host moduleを使うfixtureは`host/`へ自己完結したsourceを置き、network、global package cache、user credentialへ
依存してはなりません。fixture runnerはmanifestのtargetをdeterministic test adapterへ解決します。

converter / tooling snapshotの追加fieldは、意味を正本で定義してからschemaへ加えます。

repository rootの`.vscode/settings.json`もinventoryから生成し、contract-only sourceだけをroot workspaceの通常Seseragi
document/watcher集合から外します。fixture rootを単独で開いた場合の解析は維持し、fixture directory名をLSPへhard-codeしません。
