# Examples / Releases verification (#703)

Verified 2026-10-01. The existing `/examples/`, `/releases/` and `/ja/` mirrors
retain their page IDs and standalone layouts. This evidence is author/agent
self-review; it is not first-time-reader acceptance or a browser-layout pass.

## Examples: source and execution

The page now selects three existing canonical sources, without copying their
programs into page prose or creating a second example catalog:

| Page task | Canonical source | Exact stdout |
| --- | --- | --- |
| Shipping total | `apps/site/examples/comparisons/shipping-total/seseragi.ssrg` (#698) | `3700, 5000\n` |
| Keep an original record while updating a score | `apps/site/examples/src/language/reader-records.ssrg` | `10\n42\nAki\n` |
| Compare three function-composition forms | `apps/site/examples/src/language/reader-pipelines.ssrg` | `7\n7\n7\n` |

Each source was saved independently as `main.ssrg`, outside a repository or
manifest package, then linted and run with the published Linux v0.61.19 CLI
and Bun 1.3.9. The text's suggested edits were also executed: subtotal 4999
prints `5499, 5000\n`; score 99 prints `10\n99\nAki\n`; replacing all three
pipeline inputs with 4 prints `9\n9\n9\n`.

All three Playground links decode to the exact canonical source bytes. Those
decoded sources also compile through the committed WASM compiler and execute
through the existing Playground runtime with the same output. This exercises
the source/compile/runtime boundaries without opening a browser. The comparison
source and metadata are reused from #698.

The record wrapper changed during #705 to a single template-string `println`.
The Examples prose follows that source and explains interpolation/newlines;
it no longer teaches `do` or `_ <-` for the record example. Its former
commit-pinned source link was removed because that older wrapper would no longer
match the panel. The exact-source Playground link remains available. The pipeline
example still uses `do`/`_ <-`, so those operations are explained beside it.

The retained pipeline source link was fetched successfully at immutable commit
`ab42da841d76d322584fac096c249d586ae539ec`, and matches the displayed source:
<https://github.com/KentaroMorishita/seseragi/blob/ab42da841d76d322584fac096c249d586ae539ec/apps/site/examples/src/language/reader-pipelines.ssrg>.

The page identifies a starting task, required familiar concepts, inputs, key
operations, outputs, and related Reference destinations. Web projects are a
separate later step. Setup remains owned by #699; no installer is duplicated.

## Releases: verified public facts

The GitHub Releases API was read during implementation on 2026-10-01:
<https://api.github.com/repos/KentaroMorishita/seseragi/releases/latest>.
It returned [v0.61.19](https://github.com/KentaroMorishita/seseragi/releases/tag/v0.61.19),
published 2026-09-30, with tag commit `a8641b5a81a4`.
The [tag-fixed changelog](https://github.com/KentaroMorishita/seseragi/blob/v0.61.19/CHANGELOG.md)
matches the release notes. The page summarizes the verified collection/type
compatibility, namespace re-export, and documentation fixes. It warns that an
invalid assignment formerly accepted can now fail, without promising general
backward compatibility.

The release response confirms all twelve linked items: native archive,
matching `.sha256`, and platform VSIX for each of `darwin-arm64`, `darwin-x64`,
`linux-x64`, and `win32-x64`. Windows uses `.zip`; the others use `.tar.gz`.
The Linux archive's checksum and execution were already verified in #699.
Other platforms' presence and procedures were checked, not their execution.

The page explicitly dates its release snapshot, links the complete release
list, and distinguishes a published `release` binary from a `development`
build with the same numeric version. Main-branch documentation edits are not
presented as a newer CLI release. No release, version bump, deploy, or remote
state mutation was performed for this work.

## Checks and remaining acceptance

- `apps/site/tests/examples-releases.test.ts` verifies independent CLI lint/run,
  suggested edits, exact-source Playground decode/WASM/runtime execution, and
  four actual production-component EN/JA renders. Render checks compare all
  code/output panels and official download URLs. No second page-content
  renderer is used.
- Focused TypeScript/Biome, Seseragi formatting and `git diff --check` are run
  for changed sources/tests. Shared checks are integrated by the parent.
- The optional `SITE_EXAMPLES_RELEASES_OUTPUT` destination saves the four
  rendered HTML pages for reading review. `target/site-examples-releases-review`
  contains the implementation-review output, not a deployment or a full site.
- `examples-releases-browser.ts` adds desktop/mobile checks at 320, 390 and
  1280 pixels: both locales, static/no-JavaScript content, exact code and link
  payloads, code legibility/scrolling, page overflow, and navigation to first run.
  These browser checks were **not run** in this environment.
- Full-site rendering, desktop/mobile visual acceptance, screenshots, and
  actual first-time-reader feedback remain **unverified**. The known full-site
  render memory limit and browser restriction were respected. Source/runtime
  checks do not close these remaining acceptance requirements.

## Independent reading review

On 2026-10-01, a second agent read all four saved EN/JA HTML pages without
editing their sources. It reported no blocking task/prerequisite/notation,
output, or release-installation ambiguity. It confirmed that the record wrapper
matched the displayed source. One optional Japanese release sentence was made
more direct: the fix concerns incorrectly accepting values with incompatible
type arguments. The release source supports that correction. The focused
render was refreshed afterward.

This was an independent, first-use-simulating text/HTML review, not feedback
from an actual TypeScript reader and not desktop/mobile visual inspection.
