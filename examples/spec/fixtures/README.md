# Conformance fixtures

このdirectoryは言語機能を機械的に網羅するfixtureです。教材としての読みやすさより、一機能一期待結果と
境界条件の明示を優先します。

- `compile/`: parseとtype checkに成功すべき最小source。
- `diagnostics/`: 失敗すべきsourceとexact diagnostic code / span。
- `projects/`: module、package、foreign binding、`.d.ts`変換など複数fileが必要なcase。

fixtureだけから意味規則を推測せず、必ず `docs/spec/` の節を `../COVERAGE.md` から参照します。
