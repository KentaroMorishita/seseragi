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

- `phase`: `compile`、`diagnostic`、`run`、`convert`、`tooling`のいずれか。
- `spec`: 根拠となる正本section。
- `lock`: `generate`または`fixture`。`generate`はrunnerがprojectをtemporary directoryへcopyし、offline resolverで
  lockfileを生成してから検証する。repository内へ生成物を書かない。`fixture`はproject内の`seseragi.lock`をそのまま
  使用し、更新しない。
- `stdout`: run fixtureのexact UTF-8 / LF snapshot。末尾newlineを含む。

host moduleを使うfixtureは`host/`へ自己完結したsourceを置き、network、global package cache、user credentialへ
依存してはなりません。fixture runnerはmanifestのtargetをdeterministic test adapterへ解決します。

diagnostic projectはsingle-file fixtureと同じdiagnostic objectへ`file`を追加し、project rootからの`/`区切りrelative
pathを指定します。converter / tooling snapshotの追加fieldは、意味を正本で定義してからschemaへ加えます。
