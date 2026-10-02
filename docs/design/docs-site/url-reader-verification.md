# URL reader verification

Status: bounded authoring, executable evidence and independent complete-body
review are complete. Whole-site integration, documentation-page browser layout,
publication and genuine first-time-human feedback remain separate gates.

## Scope and provenance

Twenty existing std/web/navigation identities have forty EN/JA bodies using the
existing typed Block renderer. The scope is in `url-reader-work-item.md` and
`apps/site/scripts/url-reader.ts`. No reference route, canonical signature,
declaration reading, kind label, compiler or runtime behavior is changed.

The verified toolchain is official Seseragi 0.61.19 (release commit a8641b5a81a4,
Linux x64, Unicode 17.0.0), Bun 1.3.9 and TypeScript 5.8.3. The thirteen canonical
Seseragi sources and thirteen ordinary TypeScript counterparts use identical
shown inputs and thirty-three matching printed observations.

## Browser target without live navigation

The canonical module is browser-only. Sources live in the separate web-target
`apps/site/examples/projects/url-reader` package, with its own checked lock.
They are not imported into `apps/site/examples/src/main.ssrg` or the existing
process-target aggregate. The process SSG reads example source as data.

Every canonical program builds as its own isolated official web-target package.
The checked-in package also builds for web, preserves its lock, and rejects a
process run with SES-K0203 / provider.target-mismatch. The exact-source committed
WASM entry contracts have Console only and no provider entries. The examples
calculate values and print; they never read or change browser location/history.

A standalone-file process invocation happened to work during feasibility. Its
compile path differs from package target validation. That bounded observation
does not change the canonical target or justify documenting process support.

## Focused checks

The final focused command passes **10 tests, zero failures, 2,782 assertions**:

```
bun test apps/site/tests/url-reader.test.ts apps/site/tests/url-reader-editorial.test.ts
```

Use the verified release compiler through `SESERAGI_BIN`. Retain execution and
render artifacts with `URL_READER_EVIDENCE_DIR` and `URL_READER_RENDER_DIR`.

- Thirteen official isolated web-package builds, plus the checked-in package
- Thirteen strict-TS executions and exact-source committed-WASM executions with
  matching output and Console-only entry requirements
- Seven invalid fixtures, each with one intended SES-T0101 diagnostic and a
  canonical working example showing the corresponding repair
- Fifty-two strict-checked current-runtime assertions covering duplicate order,
  updates returning new values, percent/query encoding, absent/empty values,
  relative bases, offsets and fragment decoding boundaries
- Exact WASM probes for the UTF-16 query offset and malformed-UTF8 fragment error
- All forty complete production bodies checked for source/output panels,
  canonical declarations, source payloads, title-labelled links, locale identity
  and within-module previous/next navigation
- Exact identity, owner, namespace and kind collision controls, with deferred
  navigation pages retaining their prior state
- All 84 owned Seseragi sources/fixtures pass canonical formatting; focused site
  host typechecking, strict comparison/runtime typechecking, Biome and diff checks
  pass

The seven mistakes are passing String or Either directly to renderUrl, reversing
withQuery's Query/Url arguments, calling the emptyQuery value, using the
NavigationError formatter on UrlBuildError, accessing Query as a record, and
returning a structural record as Query. Their precise diagnostics are retained;
no fabricated error wording is presented as compiler output.

## Fair comparisons and relevant limits

Native URL and URLSearchParams are already short, capable APIs. Normal examples
use them directly on the shown valid inputs. The UrlBuildError comparison adds
HTTP(S), userinfo and percent-triplet policy checks explicitly because it compares
those failure classifications. The extra checks are not imposed on the entrance.

Source and observed probes retain these important distinctions:

- Repeated names and full entry order survive query parsing, inspection and append
- set replaces at the first old position and removes later same-name pairs; remove
  clears all matches; missing values differ from present-empty values
- Query encoding uses form-style spaces as +, while fragments preserve a literal +
- Query extraction/rendering can normalize spellings such as %20 and ~
- parseQuery's doubled-leading-question-mark behavior differs from a plain native
  URLSearchParams string constructor
- Percent error offsets are UTF-16 code-unit indices; parseQuery computes them
  after removing its first leading question mark
- Well-formed percent triplets do not guarantee valid UTF-8. A parsed #%FF
  fragment reaches urlFragment and throws URIError, outside Left/Nothing; query
  decoding instead uses a replacement character

Those limits appear after useful cases on the relevant pages. They do not become
an unexplained wall before a beginner's first result. No runtime/compiler repair
or public defect report is part of the work.

## Complete-body review and corrections

Independent reviewers read all twenty English and all twenty Japanese complete
rendered bodies, including both complete source panels, output, declarations,
target instructions and related links. They did not fill gaps from source code or
other articles.

The initial review identified two English phrases that attributed printing to
pure value-producing functions, and twelve Japanese pages lacking a local
explanation of the match wildcard in Left _. The corrections distinguish
rendering from println and explain that the wildcard ignores the failure reason.

The same English wording improvement was made on urlFragment. Japanese
queryValues now explicitly distinguishes the comprehension's string-array result
from literal separators in the displayed output.

Exactly sixteen bodies changed: three English and thirteen Japanese. Every
changed body was reread in full; the remaining twenty-four bodies are
byte-identical. All forty corrected bodies pass this bounded independent review.
Final exact-source rendering preserves the reviewed full HTML and article hashes.
This is model-reader evidence, not genuine first-time-human acceptance.

## Actual public Playground evidence

Actual public-browser execution covers exactly three canonical source seeds:

- share-search-link: `1f79418621f5d6575557ab9c9bfc488c23ddadf68a367ed989c51dea0ed79a59`
- read-selected-tags: `278c94ee60020340e0519353d1625a430e44d941753446fe7ef7db9209590013`
- url-build-errors: `f273fdc2246d0ea728a40b9f67b54aad35e899da0da137c79e526aa9a1b2c688`

For each, the source URL and actual editor text matched the canonical hash, Run
completed with exact expected output, the page URL stayed unchanged, and no site
console errors or warnings were observed. The observed public asset was
playground-CcCLbc7e.js. An exact deployed compiler/runtime version or commit was
not independently exposed or verified.

Screenshots were emitted during the checks but no saved screenshot file is
claimed. This evidence does not cover every other seed, documentation-page
layout, provider/history behavior, exhaustive browser engines or deployment
correspondence. The malformed-UTF8 probes ran in the committed-WASM/current-runtime
harness, not on the public Playground.

## Retained evidence and remaining gates

Exact source, complete body and full-HTML hashes, focused logs, diagnostic results,
web artifacts, target checks, independent review histories and public-browser
records are retained. The review ledger appendix preserves its entire prior
prefix and all 106 original checkbox rows; no earlier conclusion is replaced.

No commit, push, deployment or whole-site integration is claimed by this bounded
verification. Those steps, the full catalog/layout checks and real first-time
reader feedback require separate evidence.
