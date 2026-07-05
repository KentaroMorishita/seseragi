# Project fixtures

一fileで表せないmodule graph、manifest、foreign binding、generated source、lockfileのcaseをdirectory単位で
置きます。各projectは独立した `seseragi.toml` を持ちます。

各project rootは`project.expect.json`を持ちます。

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
- `command`: diagnostic phaseの実行surface。省略時は`compile`、converterなら`convert`、tool protocolなら
  `tooling`を指定する。
- `artifacts`: convert / tooling phaseの生成物。`output`はtemporary project内の生成先、`snapshot`はrepository内の
  exact expected textです。同じoutputを二度宣言できず、snapshotはUTF-8 / LF / final newlineを持つ。

host moduleを使うfixtureは`host/`へ自己完結したsourceを置き、network、global package cache、user credentialへ
依存してはなりません。fixture runnerはmanifestのtargetをdeterministic test adapterへ解決します。

converter / tooling snapshotの追加fieldは、意味を正本で定義してからschemaへ加えます。
