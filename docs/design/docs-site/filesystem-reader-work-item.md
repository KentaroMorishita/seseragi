# Local work item: paths and small UTF-8 reports

Status: local authoring, focused verification and bounded independent reading
are complete. This record does not establish publication, first-time-reader
acceptance, or whole-site integration.

## Reader task

An everyday TypeScript developer names a report file, creates it without
replacing an existing file, reads strict UTF-8 text, chooses replacement or append
when intended, and uses a temporary workspace for intermediate files. Explain
ordinary function calls, named alternatives, match, and Effect execution locally;
a Tour, FP background or TypeScript discriminated-union knowledge is not assumed.

## Exact existing scope

The batch covers sixteen identities, in English and Japanese:

- std/path module, Path, parse, render and child
- std/fs module, FileSystem, FileSystemError, FileTextError, WriteMode,
  CreateNew, Replace, Append, readTextUtf8, writeTextUtf8 and
  withTemporaryDirectory

These are two existing module pages and fourteen existing API leaves. Their
canonical identities, owner, namespace, item kind, declarations and route names
stay unchanged. No future application route or unselected API is added or counted.

## Example and comparison boundaries

Filesystem examples are complete process-target programs. Each uses its own
managed temporary directory, and explains the FileSystem service requirement.
A successful browser build does not supply that service; no unverified browser
or Playground launch link is provided for filesystem work.

TypeScript comparisons use direct Node filesystem operations and ordinary
functions. The flags wx, w and a already express the three write modes concisely.
The strict text comparison uses TextDecoder with fatal:true and ignoreBOM:true:
default UTF-8 string decoding has different malformed-byte behavior. A plain
try/finally temporary helper does not have identical simultaneous-failure or
cancellation semantics.

The beginner story uses verified success and ordinary typed failure with
successful cleanup. Advanced combined callback/cleanup failure behavior must
remain qualified: a direct callback-error result is different from a later
provider shutdown failure. Do not promote a source-specification diagnostic
promise into an execution claim without evidence, or repair runtime behavior
within this documentation task.

## Deliverables and acceptance

- Japanese-first typed locale copy and exact-identity editorial dispatch
- Canonical complete examples, expected observations and strict TS counterparts
- Focused execution, rejected-input/correction and complete-body render checks
- Exact declaration/source/output and same-identity locale/link protection
- Full English/Japanese reading review, with corrected bodies reread completely
- Separate records for execution, rendering, reading, actual browser layout,
  first-time-reader feedback, integration and publication

The existing semantic Block renderer remains the sole article format. Shared
catalog/build/example registrations and lock updates belong to integration.
Existing authored pages and historical reader-review records remain intact.

## Local completion

All sixteen identities now have paired explanations, complete runnable sources
and exact guarded dispatch. Final focused checks pass twelve tests and 1,920
assertions. All 32 full bodies were read independently; fourteen corrected
bodies and the two final small revisions were reread completely. No substantive
reader finding remains. Existing canonical declarations, example panels, links
and unrelated generated readings are preserved. Full evidence and limits are
recorded in filesystem-reader-verification.md.

Integration owns shared lock freshness, normal site gates and any
publication. Generic declaration wording for unselected zero-parameter structs
remains a separate audit item; no nearby page is counted as reviewed from a
negative-control check.
