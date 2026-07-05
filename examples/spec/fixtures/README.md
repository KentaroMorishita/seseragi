# Conformance fixtures

このdirectoryは言語機能を機械的に網羅するfixtureです。教材としての読みやすさより、一機能一期待結果と
境界条件の明示を優先します。

- `compile/`: parseとtype checkに成功すべき最小source。
- `diagnostics/`: 失敗すべきsourceとexact diagnostic code / span。
- `projects/`: module、package、foreign binding、`.d.ts`変換など複数fileが必要なcase。

fixtureだけから意味規則を推測せず、必ず `docs/spec/` の節を `../COVERAGE.md` から参照します。

single-file fixture `name.ssrg` は隣に `name.expect.json` を必ず持ちます。共通fieldは次です。

```json
{
  "schema": 1,
  "kind": "compile",
  "spec": ["2.3", "4.9"]
}
```

`spec` は根拠となる正本のsection番号です。compile fixtureは `kind: "compile"`、diagnostic fixtureは
`kind: "diagnostic"` と `diagnostics` を持ちます。rangeは0-based・end-exclusive UTF-8 byte offsetです。

```json
{
  "schema": 1,
  "kind": "diagnostic",
  "spec": ["12.13"],
  "diagnostics": [
    {
      "code": "SES-T0001",
      "severity": "Error",
      "primary": { "start": 17, "end": 21, "text": "todo" }
    }
  ]
}
```

`text` はreview用anchorで、checkerはsourceの指定byte rangeと完全一致することを検証します。compiler conformance
runnerはcode、severity、rangeを比較し、message文言は比較しません。related labelやfixを規範化するfixtureでは
同じsidecarへ `labels` / `fixes` を追加します。
