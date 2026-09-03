# Pinned Unicode data

`manifest.json` is the toolchain's Unicode-version source of truth. It pins the
official Unicode 17.0.0 UCD files and their SHA-256 digests. `ucd/` includes the
upstream sources and normalization / extended-grapheme conformance tests so
generation and verification work offline.

- `bun run unicode:generate` verifies the vendored sources and regenerates the
  TypeScript tables, lightweight runtime version, Rust version constants, and
  runtime license notice.
- `bun run unicode:check` verifies both source digests and generated freshness.
- `bun scripts/generate-unicode.ts --download` explicitly downloads the pinned
  files. Normal build and check commands never download Unicode data.

Updates must change the manifest, compiler Unicode dependency pins, generated
projections, artifact metadata and locks together, then pass the official suites
and the artifact/runtime mismatch tests. The runtime does not delegate Unicode
semantics to host ICU, `Intl.Segmenter`, Unicode regular expressions, or native
string normalization/casing. The shared property tables are also the foundation
for a future portable Regex implementation; this does not implement Regex.

Sources: [UCD 17.0.0](https://www.unicode.org/Public/17.0.0/ucd/),
[UAX #15 revision 57](https://www.unicode.org/reports/tr15/tr15-57.html),
[UAX #29 revision 47](https://www.unicode.org/reports/tr29/tr29-47.html).
Data and derived tables are distributed under [Unicode License v3](LICENSE).
