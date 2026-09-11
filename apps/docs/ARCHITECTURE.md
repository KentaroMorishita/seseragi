# Official Docs: content and application contract

O03 phase A (#591), audited against released main
`46b6e0cdd8cdee67d4a74f7add0c1e99e812b367` (v0.61.9).
This is the implementation contract for subsequent leaves, not a claim that the
site or the proposed pipelines have been implemented.

## Sources and ownership

| Surface | Existing source | Docs responsibility |
| --- | --- | --- |
| Language meaning | [canonical spec](../../docs/spec/README.md) | Explain and link; never redefine semantics |
| Installation and local Web | [Getting Started](../../docs/GETTING_STARTED.md), [README](../../README.md) | A continuous installation → run → test → build journey |
| Executable examples | [examples/spec](../../examples/spec/README.md) | Reference source files and verify snippets against them |
| Ordered learning | [Tour curriculum](../../scripts/tour-curriculum.ts), [lessons](../../scripts/tour-lessons.ts) | Link to Tour; do not duplicate its ordered progression |
| Articles | [Deep Dive catalog](../../scripts/deep-dive.ts) | Preserve unordered Seseragi Articles at /deep-dive/ |
| Standard API | [analysis Reference consumer](../playground/src/ui/reference-browser.ts), [standard surface](../../crates/seseragi-semantics/src/prelude/surface.rs) | Generate symbols/signatures from compiler data |
| Public module API | [typed interface](../../crates/seseragi-semantics/src/model/typed_interface.rs), [syntax interface](../../crates/seseragi-syntax/src/interface.rs) | Preserve resolved identities, constraints and instances |
| Production contract | [O02 baseline](../../docs/production/first-party-baseline.md) | Measure generator, published pages and client artifacts separately |
| Provider authoring | [provider guide](../../docs/PROVIDER_AUTHORING.md) | Link implementation details from Internals |

The O02 report retains its original implementation measurement date/version.
Release acceptance is recorded in #525/#554; do not relabel historical numbers as
fresh Docs measurements. Installed CLI/LSP/extension 0.61.9 synchronization was
verified on 2026-09-11 before starting O03.

## Information architecture

Docs navigation and prose are Japanese initially; identifiers and route segments
are stable English. A locale dimension can be added later without generating
empty translations now. Routes below are relative to a configurable site base.
The deployment origin and base must be explicit build inputs, never inferred
from the developer machine.

| Section | Route prefix | Reader task |
| --- | --- | --- |
| Getting Started | /getting-started/ | Install, Hello World, project, run/test/build, Web and production |
| Language Guide | /language/ | Values, declarations, functions, application, ADT, collections, modules, traits, Effect, Signal, failures |
| Application Guide | /applications/ | Web, HTTP, JSON, filesystem, process, time, storage and databases |
| Concepts | /concepts/ | Expressions, world boundaries, providers, reactivity, pure Html and failure values |
| Cookbook | /cookbook/ | Forms, decoding, validation, concurrency/resources, transactions and tests |
| Reference | /reference/ | Find compiler-owned symbols and public contracts |
| Internals | /internals/ | Compiler pipeline, Core IR, backend and provider architecture |

The home page offers these entry points. Tour, Playground, Articles and Spec are
named external surfaces, not extra nested guide sections. No new spec or copied
API signature becomes authoritative through publication on Docs.

## Content → page contract

Author prose under `apps/docs/content/`; keep Seseragi rendering/components under
`apps/docs/src/`. Generated files go into ignored build output, never hand-edited
source. The concrete content encoding is selected during the shell leaf after
static-output proof; Markdown is a preferred authoring input, not a requirement
for an unimplemented Seseragi Markdown parser.

Each page has a stable id, unique route, title, summary, section and ordered
content blocks. Each executable example references a repository-relative
canonical source path plus an optional named/validated range. The pipeline reads
that source directly and records its digest; it must not maintain a second code
string. Missing sources/ranges, duplicate ids/routes and unsafe output paths are
build failures. Narrative-only pseudocode must be visibly labelled and cannot
be advertised as executable.

All public paths use trailing-slash routes mapped to `route/index.html`. Reject
absolute filesystem paths, traversal, query/fragment-bearing route definitions
and collisions before writing. Internal links derive from the same route table;
unknown internal targets fail validation. Link generation must honor a non-root
base, including assets, search results and navigation.

The page model distinguishes plain text, code and structured markup. Escape text
once through pure Html. Do not pass authored or API text through raw HTML string
concatenation. Full-document serialization must include doctype, language, head,
title/description and a single main landmark; SSR of a body fragment alone does
not prove an accessible document.

## Decision evidence: generation and client boundary

Existing [pure Html lesson](../../examples/spec/lessons/27-pure-html.ssrg) proves
`html.renderToString` without Dom. Current CLI supports process/web builds and
release artifacts. Start phase B by building and executing a Seseragi process
program that renders multiple pages. Compare development/release output, escape
behavior and deterministic repeated runs. Reuse canonical build manifests for
the generator; do not call a generator manifest a manifest of the final website.

A new `ssg` target and a web prerender phase are deferred: neither is necessary
merely to invoke an existing process program at build time. A small host script
may validate inputs, invoke the canonical CLI and publish files. Page composition
and HTML rendering remain Seseragi application responsibilities. A reproduced
limitation determines whether a compiler/build leaf is needed.

No client bootstrap is emitted for a page needing only content and links.
Search, navigation, copy and examples require explicit enhancement roots and
separate measured client artifacts. Do not imply that renderToString preserves
handlers or provides hydration. Browser behavior and production retention must
be measured before choosing attachment boundaries.

## Generated Reference contract

Reuse compiler-owned standard analysis metadata and typed public interfaces;
do not parse signatures from prose or maintain a handwritten symbol registry.
Keep stable module/symbol identities, signature/constraint relationships,
instances, source provenance and metadata schema version. Documentation text,
since and deprecation require a field-by-field producer audit: absent data is
unavailable, never a fabricated version or an inferred description. Project
linking interfaces alone do not prove a complete documentation export command.

The Reference leaf must demonstrate a real producer → JSON → page pipeline and
freshness failure after a public API change. If the existing CLI lacks export,
add the narrow compiler-owned tooling boundary with fixtures; copying checked-in
analysis fixture JSON is not a current Reference pipeline.

## Ordered lookahead and verification

1. B: process multi-page proof; select canonical SSG contract from actual results.
2. C: shell, page/route generation and responsive navigation using that contract.
3. D: generated Reference with metadata provenance and freshness validation.
4. E: human-facing baseline linked to executable canonical sources.
5. F: selective search/copy/highlighting/Playground behavior with browser proof.
6. G: link, SEO, accessibility, mobile and large-corpus regression gates.
7. H: production/deploy reproducibility, measured costs and O04 handoff.

Only audited work is converted to queue leaves. Phase A verification checks the
source inventory and self-reviews ownership, routes, escaping and truthful
future boundaries. Subsequent implementation checks include source drift,
route rejection, deterministic generation and real browser behavior. Terminal
Promotion owns the combined integration/deploy gates under #512/#555. Phase A
alone does not require a compiler build or canonical CLI release.
