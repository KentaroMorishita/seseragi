# Bytes inspection fixtures

These are test inputs and baseline expectations, not an assertion that a WIP
candidate has passed.

- `prior-authored-pages.json`: 344 exact routes returned by the real typed
  authored-library dispatcher for the prior completed candidate at
  e28d17566dcff7fe5d81f9b3ad9660516bebab47. The projection removes only duplicate
  locale entries, not identities. The new eleven routes must be disjoint
- `unchanged-reading-hashes.json`: all 1,811 canonical declaration readings
  except std/bytes::BytesSliceError, captured before the new adapter was wired
  and checked against the prior complete reference snapshot
- `unselected-controls.json`: thirty localized std/bytes controls from that
  candidate's complete production render. API controls cover the full article;
  module controls stop before the generated public-symbol list
- `boundaries.ssrg` / `.stdout`: 31 exact numeric-byte, index, range, chunk-order,
  empty-input, unchanged-input and public-error-constructor observations
- `native-differences.ts`: native TypeScript typed-array wrapping/truncation and
  slice normalization, checked separately rather than claimed equivalent
- `invalid/`, `repairs/`, `negative-cases.json`: six precise type/export mistakes
  and complete working repairs; expected diagnostics are checked independently

Current exact displayed source bytes live in examples/src/bytes-inspection and
are compiled/executed from an isolated temporary directory during tests. The
six TypeScript comparisons use the same source on Node and Bun. WASM host tests
are distinct from actual-browser or deployed-Playground tests.
