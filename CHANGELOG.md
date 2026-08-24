# Change Log

## [0.21.1] - 2026-08-24

- Connected the normative `std/effect::parallel` surface to unbounded
  structured execution with input-order results, deterministic same-turn
  failure selection, and sibling cleanup before completion.
- Preserved concrete element inference when a collection receives a nested
  generic expected type containing holes, allowing arrays of Effects to infer
  their environment, failure, and success types.

## [0.21.0] - 2026-08-24

- Added structured `Fiber<E, A>` concurrency with scoped child supervision,
  join, await, poll, interrupt, race, cooperative yield, and bounded parallel
  traversal that preserves input order.
- Added one-shot Deferred values and bounded or unbounded FIFO Queues with
  cancellation-safe waiters, non-blocking operations, and deterministic close
  draining.
- Added FIFO Semaphores with owned, idempotent Permits and cancellation-safe
  `withPermit`, and connected Lesson 16 plus normal CLI and runtime conformance
  fixtures to the public standard surfaces.

## [0.20.0] - 2026-08-24

- Added the official `seseragi/sqlite` package with file and in-memory
  databases, parameterized query and execute operations, SQLite-specific
  values, and typed row decoders.
- Added `BEGIN IMMEDIATE` transaction composition with commit on success,
  rollback on typed failure or cancellation, deterministic cleanup, and typed
  busy/locking failures.
- Connected the Bun built-in SQLite driver through the process-only
  `seseragi/runtime-sqlite#bun` Provider, with browser target rejection before
  entry evaluation and no universal database abstraction.

## [0.19.0] - 2026-08-24

- Added the official `seseragi/postgres` package with parameterized queries,
  PostgreSQL-specific values, typed row decoders, and command-aware results.
- Added scoped pool, transaction, and cursor lifecycles with commit on success,
  rollback on typed failure or cancellation, and deterministic connection
  cleanup.
- Connected the stable PostgreSQL Provider service to ordinary CLI projects
  through the bundled `pg` and `pg-cursor` adapter without exposing host driver
  objects or implementing a wire protocol.

## [0.18.0] - 2026-08-23

- Added the canonical `seseragi.lock` schema-1 reader and writer with exact
  workspace, path, and registry identities, dependency edges, and SHA-256
  manifest and content digests.
- Added explicit `seseragi lock update`; project run, build, and development
  now reject missing or stale locks with `SES-K0102` without rewriting them.
- Integrated exact Provider package, artifact, ABI, target, feature, and host
  package selections into the same machine-independent project lock contract.

## [0.17.0] - 2026-08-23

- Added `std/http/server` request accessors and explicit empty, Bytes, text,
  and JSON response constructors with custom status and headers.
- Connected `Handler<R, Never>` to independent per-request Effect executions,
  captured environments, concurrent handling, and close-driven cancellation
  with exactly-once resource cleanup.
- Added Bun and Node HTTP server providers plus a normal Seseragi `POST /users`
  E2E covering Bytes, UTF-8, derived JSON codecs, typed recovery, and JSON
  responses.

## [0.16.0] - 2026-08-23

- Implemented restricted Effect and Stream requirement merge across parsing,
  module interfaces, semantic types, Core IR, and TypeScript type lowering.
- Normalized generic requirement merges after substitution with empty-record
  identity, deterministic field union, and same-field deduplication.
- Added diagnostics for invalid merge positions, optional or non-record
  operands, and conflicting same-named service fields.

## [0.15.0] - 2026-08-23

- Defined `std/http/server::Handler<R, E>` as the application-facing
  Effectful request handler while requiring explicit typed-failure recovery at
  the `Handler<R, Never>` server boundary.
- Composed handler requirements into the server startup environment without
  exposing Provider identities, host request objects, or Promise details.
- Fixed one child scope per request, concurrent handler independence, and the
  cancellation, cleanup, late-response, defect, and listener-failure
  boundaries for the implementation slice.

## [0.14.0] - 2026-08-23

- Added lexical `std/effect.scoped` and `acquireRelease` resource ownership
  with LIFO finalization on success, typed failure, defect, and cancellation.
- Closed the acquire-to-registration cancellation gap and kept finalizers
  uninterruptible, exactly once, and able to perform Effect operations.
- Unified filesystem, HTTP server, and PostgreSQL Provider resources with the
  same scope mechanism while preserving explicit idempotent close behavior.

## [0.13.0] - 2026-08-23

- Added canonical `DomContent` leaf bindings for text, attributes, form
  properties, and styles without rebuilding static sibling DOM.
- Added mount-owned structural regions with nested subscription and event
  cleanup while preserving region boundaries and matching hydrated nodes.
- Preserved stable Signal transaction values, `distinct` write suppression,
  controlled-input IME composition, selection, and unmount cancellation across
  CLI Web builds, Playground runtime, and actual Chromium execution.

## [0.12.0] - 2026-08-23

- Added the opaque `DomMount` lifecycle with explicit mount, await, idempotent
  unmount, root cancellation, target ownership, and clear or preserve cleanup.
- Added strict and replace hydration policies with typed mismatch paths while
  preserving matching initial server DOM identity and preventing partial mounts.
- Rebased Web UI semantics around mount-owned Signal bindings instead of making
  whole-tree reconciliation the canonical update model, leaving fine-grained
  leaf and structural-region updates to the next Web UI surface.

## [0.11.0] - 2026-08-21

- Added `std/signal.distinct` as an Eq-constrained derived Signal that keeps
  the source's current value and suppresses Eq-equal publications.
- Preserved transaction and glitch-free graph semantics through composed
  `map`, `combine`, and `switchMap` nodes without stopping source updates.
- Added custom-Eq lowering, Analysis and runtime ABI artifacts, canonical
  execution coverage, and actual Playground WASM execution.

## [0.10.0] - 2026-08-21

- Added browser-targeted `std/web/storage` with explicit `Local` and `Session`
  areas, String key/value operations, `Maybe` lookup presence, and copied,
  sorted key snapshots.
- Classified quota, security, and unavailable host failures at the Provider
  boundary without exposing browser `Storage` objects or implicit JSON
  conversion.
- Added canonical Provider/runtime ABI artifacts, process-target diagnostics,
  WASM lowering, and Playground execution coverage for explicit JSON storage
  round trips.

## [0.9.4] - 2026-08-21

- Rejected malformed nested lambdas before recovery HIR can reach TypeScript
  lowering and emit an unbound `_` placeholder.
- Kept namespaced calls to non-exported trait-backed collection functions as
  explicit `SES-N0104` diagnostics while preserving Prelude as their SSOT.
- Added CLI project, WASM, Playground, conformance, and actual Array/List
  `reduce` execution coverage for canonical curried lambda syntax.

## [0.9.3] - 2026-08-21

- Formalized the v0.9.2 logical-condition fix as an executable language
  contract for `if` branches containing `&&` and `||`.
- Covered three-term chains, parenthesized mixed operators, nested `if` and
  `match` branches, direct Bool results, and skipped failing right-hand sides.
- Verified the same generated TypeScript and runtime results through CLI
  projects, WASM, and Playground execution.

## [0.9.2] - 2026-08-21

- Preserved grouped arithmetic semantics in generated TypeScript by rendering
  binary children with operator precedence and associativity awareness.
- Covered additive, multiplicative, exponentiation, comparison, logical, and
  conditional expression nesting, including pipeline-connected expressions.
- Added CLI, project compiler, WASM, Playground, generated artifact, and actual
  execution regressions for the calculator ratio and average-difference cases.

## [0.9.1] - 2026-08-21

- Added canonical standard evidence for Float `+`, `-`, `*`, `/`, `%`, and
  `**`, matching the existing Int arithmetic operator surface.
- Preserved the selected `std/float::*` identity through Typed HIR and Core IR
  while lowering Float arithmetic to the corresponding TypeScript operators.
- Added generated artifact and actual execution coverage for all six Float
  operators alongside the existing checked Int regression matrix.

## [0.9.0] - 2026-08-21

- Added the browser-targeted `std/web/navigation` surface for normalized URL,
  path, ordered query, fragment, current location, and history operations.
- Connected same-origin push/replace, back/forward, and cancellable navigation
  change observation through the browser Provider boundary without exposing
  host `Window`, `Location`, or `History` objects.
- Added compiler, runtime ABI, target diagnostics, WASM, Playground preview,
  and real-browser coverage for navigation and popstate behavior.

## [0.8.0] - 2026-08-21

- Published the immutable `std/http` small-response client surface with
  validated methods, URLs, ordered headers, body limits, `Bytes` bodies, and
  typed build and transport failures.
- Connected the same Seseragi request API to the existing Bun, Node, and
  browser Provider boundary while preserving cancellation and explicit manual
  redirect handling.
- Added compiler, Analysis, Reference, runtime ABI, conformance, CLI, WASM, and
  browser execution coverage for real GET/POST request and response handling.

## [0.7.0] - 2026-08-20

- Connected the canonical `std/effect`, `std/ref`, Duration, and Clock APIs to
  normal projects across CLI, WASM, and Playground execution.
- Added typed Effect conversions and recovery, cold deferred effects, mutable
  references, and Clock-backed retry, repeat, schedule, sleep, and timeout.
- Added explicit call-site type arguments while preserving typed failures,
  defects, and cancellation as distinct Effect outcomes.

## [0.6.3] - 2026-08-20

- Added `JsonEncode` and `JsonDecode` deriving for nominal Struct, ADT, and
  Newtype declarations, including generic, imported, and guarded recursive
  codec evidence.
- Fixed canonical tagged-ADT decoding to report unknown constructors at the
  `tag` path, and shared linear strict-object lookup across structural and
  derived record decoders.
- Added executable Lesson 26, schema, project, diagnostic, and runtime coverage
  for nominal JSON codec derivation and its coherence failures.

## [0.6.2] - 2026-08-20

- Made exact Decimal-to-Int decoding reject extreme positive and negative
  exponents as typed failures without expanding unbounded decimal strings.
- Made structural record decoding linear in object fields while preserving
  strict unknown/missing-field behavior and declaration-order decoding.
- Preserved the `tag` field path when an `Either` decoder rejects an unknown
  constructor tag.

## [0.6.1] - 2026-08-14

- Added executable `std/json` parsing and canonical stringification with exact
  decimal numbers, ordered object fields, typed syntax failures, and
  path-aware decode errors without delegating semantics to host JSON APIs.
- Added Prelude `JsonEncode` and `JsonDecode` evidence for supported standard,
  tuple, and closed structural record types, plus core Decoder combinators
  across CLI, WASM, Playground Reference, and conformance surfaces.

## [0.6.0] - 2026-08-14

- Added opaque `Byte` and immutable `Bytes` standard types with validated
  construction, slicing, copying, collection operations, and copy-only
  `Uint8Array` host adapters.
- Added strict and lossy UTF-8 encoding and decoding through `std/text`, with
  typed invalid-sequence byte offsets across the compiler, runtime, Analysis,
  Reference, CLI, WASM, and conformance surfaces.

## [0.5.1] - 2026-08-14

- Made official extension packaging tests portable across Windows and Unix so
  the canonical release matrix can converge without changing tagged source.
- Allowed an explicit patch version to recover release infrastructure even
  when the fix is confined to internal verification surfaces.

## [0.5.0] - 2026-08-14

- Added a target-neutral runtime Provider contract and connected Clock, HTTP,
  filesystem, PostgreSQL, browser DOM, and lifecycle implementations across
  Bun, Node, and the Playground.
- Unified filesystem and virtual project inputs, canonical target selection,
  standard module registration, package fixtures, and Web project scaffolding.
- Added `seseragi dev` and official VS Code Run, Web Build, Dev, Stop, and Open
  Browser commands backed by the same canonical project and runtime behavior.
- Added downloaded CLI, LSP, VSIX, and browser product-journey validation for
  the Web toolchain.
- Added workspace-wide references, rename, and workspace symbols with
  namespace, alias, re-export, operator, UTF position, and overlay support.
- Reorganized Playground editor controls and shared settings, including
  adaptive formatter width propagation from UI through the WASM driver.
- Made every user-visible PR establish a SemVer and CHANGELOG boundary, and
  made pending versions converge from gated main commits to exactly one GitHub
  Release with local CLI, LSP, and official VSIX dogfood synchronization.

## [0.4.7] - 2026-08-12

- Integrated the five abstraction-design Deep Dive materials into the normal
  Tour sequence as editable lessons with exercises and compiler diagnostics.
- Recast the Deep Dive route as an unordered Articles surface for background,
  internals, and trade-offs without a separate progress or prerequisite model.
- Added data-driven middle-insertion coverage for Tour navigation, progress,
  prerequisites, and stable direct lesson URLs.

## [0.4.6] - 2026-08-12

- Made full-page visual baselines derive their pixel-difference budget from the
  Preview, Editor, or Workspace surface they are intended to protect.
- Added sensitivity checks for localized spacing, typography, and alignment
  regressions while preserving platform baselines and review artifacts.

## [0.4.5] - 2026-08-11

- Fixed stale Tour source excerpts so the Signal and Monad walkthroughs show
  the operators and forms described by their text after canonical formatting.
- Updated the matching Tour highlight expectations and reviewed 320px HTML
  source baselines used by Web UI visual regression CI.

## [0.4.4] - 2026-08-11

- Added initial Deep Dive articles for type constructors, Trait evidence and
  coherence, Functor/Applicative/Monad laws and desugaring, and custom Trait
  boundary design.
- Gave every Deep Dive article a formatted executable source, expected stdout,
  failing source, native diagnostic snapshot, Tour prerequisites, and recap.
- Added Deep Dive sources and diagnostics to the native sample gate and
  rendered the verified examples with Seseragi syntax highlighting.

## [0.4.3] - 2026-08-11

- Added an optional Deep Dive surface with independent category, chapter,
  article, route, prerequisite, and progress contracts without restoring a
  top-level Learn entry or mixing the content into Tour or Discover.
- Linked related Tour lessons and Tour completion to stable Deep Dive article
  URLs while preserving the canonical Tour curriculum and progress.
- Rejected duplicate IDs, broken prerequisites, cycles, orphan article files,
  and empty sections when generating the Deep Dive catalog.

## [0.4.2] - 2026-08-11

- Finalized the syntax-driven canonical formatter contract across imports,
  declarations, structural right-hand sides, member bodies, nested records,
  collections, applications, and operator chains.
- Kept short `do`, `match`, pure blocks, structs, records, and collections
  compact while expanding only width-overflowing syntax boundaries.
- Preserved application semantics by wrapping only at existing delimiter or
  leading-operator boundaries and allowing otherwise unsafe lines to exceed
  the target width.
- Canonicalized all Playground samples and Tour sources, refreshed walkthrough
  ranges and diagnostics, and added an idempotent full-surface formatter corpus.

## [0.4.1] - 2026-08-09

- Fixed effect contract validation for parameterized failures, explicit
  success types, and environment requirements.
- Added standard rendering for `DomRuntimeError<Never>` at process entry
  boundaries.
- Rejected unsupported process host capabilities before `run` or `build` with
  an actionable target diagnostic instead of a runtime defect.
- Moved the browser DOM and IME adapters into the official runtime package so
  standalone consumers and the Playground share one implementation.
- Added `seseragi build --target web` for self-contained static browser
  bundles with managed metadata, source maps, baseline CSS, and DOM lifecycle.
- Drained cleanup registered during Effect cancellation before the shared
  cancellation promise settles, including nested and rejected cleanup.
- Rendered sample guides and structured Tour inline content with safe Markdown
  contracts in the Playground.
- Added reviewed screenshot baselines for representative Playground Web UI
  states, with expected/actual/diff artifacts on visual regressions.
- Diagnosed eager top-level initialization through immediately invoked lambdas,
  local functions, callable aliases, higher-order calls, and inherent methods
  before generated JavaScript can reach a temporal dead zone.
- Split the Tour's tuple, Record, Struct, ADT, and pattern matching material
  into staged runnable lessons with exercises and compiler diagnostics.
- Split Array, List, Range, transformation, filtering, reduction, composition,
  and empty collection behavior into staged runnable Tour lessons.
- Split Maybe, Either, defaulting, mapping, short-circuiting, typed error
  transformation, and the Effect boundary into staged runnable Tour lessons.
- Split Effect execution, `do`, success binding, typed failure, capability
  contracts, error mapping, and value conversion into staged Tour lessons with
  executable expected-failure contracts.
- Compared local and imported instances by canonical trait and argument
  identities so import aliases cannot bypass coherence diagnostics.
- Split generic functions, type parameters, generic data, Trait constraints,
  instances, Functor, Applicative, Monad, Signal, impl, and operators into
  staged runnable Tour lessons with compiler diagnostics.
- Split Signal values, mutable state, read-only views, updates, derivation,
  Applicative transactions, dynamic switching, and handler ownership into
  staged runnable Tour lessons with compiler diagnostics.
- Split static HTML, typed props, components, links, images, events, forms,
  Signal rendering, DOM mounting, typed actions, accessibility, and state
  ownership into staged runnable and interactive Tour lessons.
- Reset both the desktop lesson pane and mobile page scroll when navigating to
  another Tour lesson.
- Reused the editor's Seseragi highlighter for walkthrough source excerpts in
  Tour lesson content.
- Added persistent desktop Tour pane resizers plus compact navigation and
  Output toggles without changing the narrow-screen layout.
- Expanded the Functor, Applicative, and Monad Tour lessons to compare named
  operations, intermediate types, operators, and `do` notation step by step.
- Added staged Signal Tour lessons that pair `signals.read` / `signals.set`
  with the effectful `*` / `:=` operators and their ownership boundaries.
- Added four-step console and seven-step Web UI Tour capstones that preserve
  runnable intermediate sources, expected results, and visible change scopes.
- Added Tour quality gates for prerequisite reachability, introduced-surface
  ordering, central-concept limits, compiler-surface coverage reporting, and
  browser navigation and interactive Web UI behavior.
- Defined the boundary between the required Tour and optional Deep Dive
  material across generic abstractions, Signals, Web UI, and applications.
- Preserved canonical public ADT identity through module aliases, re-exports,
  and nested generic positions such as `Array<Html<Action>>` and
  `Signal<Html<Action>>`.
- Canonicalized imported generic types in public struct fields so field access
  keeps the same nominal identity across CLI, project, and WASM compilation.
- Separated the Playground workspace, ordered Tour, and purpose-driven
  Discover catalog while removing the duplicate Learn surface.
- Made the canonical Hello World the explicit Playground starter and added a
  persisted Blank workspace with distinct New and Reset actions.
- Required each Web Showcase to record human-approved visual and source design
  intent before its screenshot state can become a regression baseline.
- Added the Seseragi landing page as a responsive multi-module Showcase with
  official branding, interactive code chapters, and reviewed browser states.
- Reworked the shared formatter around an 88-column syntax-driven canonical
  layout for signatures, operators, collections, blocks, and comments.
- Implemented Bool-only `&&` and `||` with comparison-before-logical
  precedence and backend-independent short-circuit evaluation.
- Introduced `<$>`, `<*>`, and `>>=` through concrete Array, List, Maybe,
  Either, and Effect Tour lessons before recovering their shared Trait
  contracts in the abstraction chapter.
- Preserved separate declarations inside TypeScript foreign blocks during
  canonical formatting.
- Preserved bodyless Trait and foreign member boundaries across multiline
  signatures, call kinds, namespaces, opaque types, and deprecation metadata.
- Preserved imported nominal identity while inferring generic derived evidence
  and contextual user-defined operator function values.
- Renamed the canonical Web UI prop from `className` to HTML-native `class`
  without retaining a compatibility alias.

## [0.4.0] - 2026-08-09

- Unified the CLI, LSP, runtime, WASM, and VS Code extension on one toolchain
  version source.
- Completed the Rust toolchain migration and removed the retired TypeScript
  compiler implementation.
- Added project-aware CLI builds and LSP workspace module graphs, formatting,
  structured diagnostics, navigation, completion, and Windows file URI support.
- Expanded the language and standard library with safe integer APIs,
  collections, structural `Show` / `Debug`, pattern binding fixes, explicit
  effect failures, Signals, and the typed Web UI surface.
- Rebuilt the Playground around persisted multi-file workspaces, project
  diagnostics, cancellable browser effects, responsive editing, and a
  dedicated staged Tour curriculum.
- Added commit, channel, target, and dirty-build metadata to CLI and LSP
  version output.
- Added reproducible release artifact names and a tag-validated GitHub Release
  workflow.
- Packages each platform's CLI and LSP in one verified native archive, preserves
  Unix executable modes, and publishes a matching SHA-256 checksum.
- Gates tag publishing on main containment and the repository-wide source gate,
  then pins every release artifact and retry to the verified commit SHA.
- Renamed the official VS Code extension to `seseragi-dev.seseragi` and added
  a non-LSP migration stub for the former extension ID.
- Attaches all platform-specific official VSIX packages and the legacy
  migration stub to the GitHub Release for direct installation.
