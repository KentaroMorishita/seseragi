# Change Log

## [0.55.0] - 2026-09-05

### Changed

- Dispatch all four comparison operators through the selected `Ord.compare`
  dictionary, including local, imported, generic, and standard instances.
- Support comparison function references and partial application; preserve
  Unicode scalar ordering and reject comparison without Ord evidence, including Float.

## [0.54.0] - 2026-09-04

### Added

- Connect canonical arbitrary-precision `Decimal` values and the complete
  `std/decimal` surface through the compiler, TypeScript runtime, Reference,
  LSP, WASM, and Playground without implicit host-number conversion.
- Preserve exact base-10 addition, subtraction, multiplication, terminating
  division, JSON numbers, and Int conversion, with explicit precision contexts,
  quantization, binary64 conversion, and all six specified rounding modes.
- Report typed parse, context, arithmetic, and conversion failures, including
  UTF-8 byte offsets, and provide Eq, Ord, Hash, Show, Debug, Zero, One, and
  arithmetic dictionaries plus canonical String foreign interop.

## [0.53.0] - 2026-09-04

### Added

- Connect the opaque arbitrary-precision `BigInt` and complete `std/big-int`
  surface through the compiler, TypeScript runtime, Reference, LSP, WASM, and
  Playground without adding literals or implicit numeric conversions.
- Preserve exact arithmetic and radix conversion beyond the safe `Int` range,
  with truncating division, exponentiation by squaring, and canonical Eq, Ord,
  Hash, Show, Debug, identity, and arithmetic dictionaries.
- Report typed parse, checked division, checked power, and narrowing failures,
  including UTF-8 byte offsets and an explicit precision-preserving foreign ABI.

## [0.52.0] - 2026-09-04

### Added

- Connect `std/bytes/hex` and `std/bytes/base64` through the canonical standard
  interface, compiler, TypeScript runtime, Reference, LSP, WASM, and Playground.
- Encode lowercase hexadecimal, padded RFC 4648 Base64, and unpadded URL-safe
  Base64 canonically; decode strict alphabets, padding, and unused trailing bits.
- Report typed decoding failures with 0-based UTF-8 byte offsets and provide
  structural Eq plus stable Show/Debug dictionaries across execution surfaces.

## [0.51.0] - 2026-09-04

### Added

- Connect the specified `std/regex` opaque type, typed compile errors, options,
  UTF-8 byte spans, captures, matching, splitting, escaping, and literal or
  callback replacement through every supported product surface.
- Implement a portable leftmost-first, greedy counted NFA without delegating
  semantics to a host regular-expression engine, including scalar progress
  after empty matches and bounded behavior for ambiguous inputs.
- Share pinned Unicode 17.0.0 properties and simple case folding with the text
  runtime, and cover native CLI, generated TypeScript, WASM, Playground,
  Reference, LSP, and actual conformance execution.

## [0.50.0] - 2026-09-04

### Added

- Connect scalar/UTF-8 text operations, `std/char`, pinned Unicode normalization,
  properties, default casing/folding, and extended grapheme APIs through the
  canonical standard interface and all supported products.
- Pin Unicode 17.0.0 data and official conformance suites; use the same version
  for compiler identifiers, normalization, casing, runtime, formatter, and LSP.
- Reject generated modules before source initialization when their Unicode data
  version differs from the runtime. Preserve the primitive String ABI and
  specify UTF-8 `lengthBytes` as O(n).
- Preserve U+FEFF through UTF-8 decoding and detached text slices; provide
  checked scalar/grapheme range errors with Eq, Show, and Debug dictionaries.

## [0.49.0] - 2026-09-03

### Added

- Connect `std/validation` with `Validation<E, A>`, `Valid`, `Invalid`, five explicit construction/conversion helpers, and canonical Functor / Applicative / Eq / Show / Debug instances. Independent errors accumulate as a non-empty list in input order; no Monad instance or implicit Either conversion is added.
- Cover Validation through native and imported-project execution, strict generated TypeScript, Analysis / Reference / LSP, and WASM / Playground parity.

## [0.48.0] - 2026-09-03

- Publish the specified `std/maybe` and `std/either` operations through the
  canonical interface, Reference, native compiler, and Playground.
- Reuse Traversable and the existing Maybe/Either Applicative dictionaries for
  shape-preserving `sequence`/`traverse`, including custom and imported evidence.
- Provide conditional Maybe Semigroup/Monoid with Nothing as identity and
  source-ordered element append; retain callable values and partial application.

## [0.47.0] - 2026-09-03

- Expose `std/collection.ReduceStep`, `Next`, `Done`, and generic `reduceUntil`
  through the canonical standard interface, CLI, LSP, Reference, and Playground.
- Stop traversal immediately at `Done`, including infinite Iterators and custom
  Iterable/Reducible evidence, without materializing collections.
- Preserve imported opaque ADT constructor identity in curried callbacks and
  support partial application and function-valued reduction accumulators.

## [0.46.0] - 2026-09-03

- Complete the specified Array/List sequence APIs, including constructors,
  short-circuit prefix operations, stable sorting, insertion-ordered grouping,
  right folds, zip/unzip, and compact chunks/windows.
- Publish `std/collection.SizeError` with Eq/Show/Debug, and connect primitive
  Int/Bool/Char/String/Unit Ord dictionaries required by sorting. String ordering
  follows Unicode scalar values; Float remains intentionally without Ord.
- Preserve first-class collection functions, partial applications, and callable
  fold accumulators across native CLI, TypeScript, WASM, and Playground.

## [0.45.0] - 2026-09-03

- Implement persistent insertion-ordered `std/map` and `std/set`, with structural
  Eq/Hash lookup, shared point updates, and removal without retained tombstones.
- Connect all collection operations, conditional Eq/Show/Debug/JSON dictionaries,
  Iterable/Reducible evidence, and Map's fixed-key Functor to the standard registry.
- Preserve inferred collection elements and partial applications across native,
  generated TypeScript, WASM, and Playground boundaries. JsonObject now exposes
  the canonical persistent Map payload.
- Initialize hash seeds before evaluating application modules, including top-level
  Map values, and preserve the manifest's full signed 64-bit seed range. Missing secure entropy fails
  during startup before any application code runs.

## [0.44.1] - 2026-09-03

- Generate the Playground runtime resolver from canonical package exports,
  including Random, Entropy, Deferred, Queue, and Semaphore, and retain browser
  host adapters and provider target boundaries.
- Export the shared Stdin error constructors in the browser runtime and detect
  module/export drift against the compiler ABI and provider manifests.
- Verify seeded Random shuffle through native CLI, WASM, and the browser editor.

## [0.44.0] - 2026-09-03

- Implemented canonical `Traversable` dictionaries for Array, List, and
  NonEmptyList, with source-order traversal and shape preservation through the
  selected Applicative evidence.
- Preserved target failure semantics, including Either short-circuiting,
  user-defined error accumulation, and cold sequential Effect execution.
- Added native CLI, WASM/Playground, generated TypeScript, Prelude registry,
  and conformance coverage for standard and user-defined Traversable and
  Applicative instances across generic module boundaries.

## [0.43.0] - 2026-09-02

- Completed the `std/non-empty-list` public surface with correctly generic
  `List<A>` boundaries while retaining the immutable persistent List-backed
  representation and non-empty invariant.
- Added conditional `Eq`, lexicographic `Ord`, ordered `Hash`, `Show`, `Debug`,
  `Semigroup`, `Functor`, `Applicative`, `Monad`, `Iterable`, and `Reducible`
  standard dictionaries for `NonEmptyList`.
- Added end-to-end compiler, runtime ABI, generated TypeScript, conformance,
  and actual-execution coverage for source order, Cartesian application,
  flatMap concatenation, iteration, reduction, and element evidence.

## [0.42.0] - 2026-09-02

- Added canonical `Hash` instances and runtime dictionaries for `Int`, `Bool`,
  `Char`, `String`, and `Unit`, available through ordinary generic evidence,
  imported functions, partial application, and user-defined instances.
- Kept user-visible `Hash.hash` pure while adding a process-local, securely
  seeded lookup adapter for the future persistent `Map` and `Set` runtime, with
  the existing fixed-seed CLI and browser manifest contract for reproducibility.
- Added CLI, WASM, Playground, runtime ABI, Prelude audit, Eq-consistency, and
  intentionally unavailable `Hash<Float>` regression coverage.

## [0.41.0] - 2026-09-01

- Unified every registered standard Prelude instance with the ordinary generic
  evidence path, including equality, arithmetic, zero and one, conditional
  collection instances, and structural tuple and closed-record equality.
- Added canonical runtime dictionaries for primitive, collection, and
  structural equality plus the previously operator-only numeric and String
  instances, so operators, named methods, and generic constraints share one
  observable instance semantics.
- Added a generated standard-instance audit matrix that classifies implemented,
  conditional, structural, missing, and intentionally unavailable specification
  surfaces and detects future registry drift.
- Added matching CLI, WASM, Playground, imported-boundary, nested-evidence,
  partial-application, local-instance, and intentional `SES-T0201` regression
  coverage with refreshed canonical conformance artifacts.

## [0.40.0] - 2026-09-01

- Unified the standard Prelude trait, method, supertrait, deriving, operator,
  and builtin-instance metadata into one canonical registry shared by semantic
  analysis, typed interfaces, artifacts, references, hover, and completion.
- Added the complete standard trait surface for equality, ordering, hashing,
  display, algebraic identities, JSON codecs, higher-kinded abstractions,
  collections, traversal, and arithmetic while preserving existing identities
  and runtime semantics.
- Lowered direct standard equality and arithmetic method calls through their
  canonical runtime ABI metadata, including saturated and partial application,
  without weakening the explicit generic dictionary boundary.
- Added diagnostics for builtin-instance overlaps and regression coverage for
  multi-parameter methods, supertraits, deriving, direct calls, generated
  analysis artifacts, and LSP hover and completion.

## [0.39.0] - 2026-08-31

- Added the embedded `.d.ts` converter and `seseragi dts convert`, including
  deterministic generated foreign modules, metadata and reports, all-entry or
  selected-entry conversion, callbacks, overload selection, generics,
  namespace/declaration merging, and precise unsupported-type diagnostics.
- Added exact foreign host identity and input/settings digests to generated
  metadata, atomic per-entry replacement, previous-output change reports, and
  pre-build `SES-F0103` rejection for missing or stale bindings.
- Connected `gen/` modules to the normal project graph and made exported nested
  foreign namespaces link and lower across modules, then verified generated
  bindings through the ordinary locked TypeScript host runtime.
- Promoted every `dts-*` project fixture to real CLI conversion evidence and
  added overload, metadata/report, entry-selection, non-update-on-error, stale
  build, and end-to-end runtime coverage.

## [0.38.0] - 2026-08-31

- Added the complete `seseragi run` option contract for process/web target
  selection, text/JSON diagnostics, signal and shutdown policy, and independent
  hash and Random seeds with invocation-over-manifest precedence.
- Unified single-file and package compile diagnostics and added one-line JSON
  runtime diagnostics for typed failures, defects, and cancellation while
  preserving their canonical exit classes and cross-language source frames.
- Connected project fixture `args` to the normal CLI route and added execution
  coverage for invalid options, target and runner overrides, compile failures,
  typed failures, defects, and source-mapped Promise rejection.

## [0.37.0] - 2026-08-31

- Added the canonical `foreign "typescript"` syntax through parsing, typed
  interfaces, lowering, generated TypeScript, and Bun, Node, and Web runtime
  execution for pure, task, value, opaque, namespace, and callback bindings.
- Added checked value codecs, cold and cancellation-aware task invocation,
  exact-identity single-flight module loads, structured `Js.Error` source
  frames, host staging, manifest-backed bare package resolution, and locked
  foreign module identities and digests.
- Promoted pure, failure-phase, single-flight, Web bare-package, callback,
  copy-boundary, opaque-handle, and source-map rejection fixtures to actual
  execution coverage.

## [0.36.0] - 2026-08-30

- Added `seseragi doc --test` with lexical module/item documentation discovery,
  stable block identities, and ordinary project compilation for check, run,
  and diagnostic-code-aware compile-fail blocks.
- Added deterministic captured Effect execution, exact stdout comparison,
  original-comment diagnostic locations, canonical exit behavior, and product
  fixture coverage without rewriting package sources.

## [0.35.0] - 2026-08-29

- Added the normal-source `std/test` tree and assertion surface with isolated
  Clock, Random, Console, Logger, and root resource scopes per case.
- Added recursive test discovery and `seseragi test` filtering, exact-name,
  parallelism, timeout, seed, stable reporting, and canonical exit behavior.

## [0.34.6] - 2026-08-29

- Made child-process `SearchPath` execution reject a missing command `PATH`
  before Node, Bun, or the host OS can apply an implicit fallback.
- Rejected unknown and malformed child-process Provider failures as boundary
  defects while preserving explicit `PATH` and direct `ExecutablePath` runs.

## [0.34.5] - 2026-08-29

- Retained SQLite and PostgreSQL rollback ownership until transaction commit or
  rollback succeeds, including typed commit failures and cancellation races.
- Kept transaction cleanup idempotent and parent-owned cleanup in reverse order
  while preserving primary and suppressed cleanup defects.

## [0.34.4] - 2026-08-29

- Fixed isolated Playground and Vercel production builds by declaring and
  locking the canonical local TypeScript runtime package dependency.
- Added an isolated frozen-install production build gate so runtime package
  imports are resolved and bundled without repository-root dependencies.

## [0.34.3] - 2026-08-29

- Fixed Stream branch ownership so partial `zip` and `merge` opens, buffered
  producers, terminal failures, and cancellation always drain cursors and
  child Effect scopes exactly once.
- Preserved cleanup-defect priority and ordered suppressed defects without
  allowing a cursor close failure to skip later resource finalizers.

## [0.34.2] - 2026-08-29

- Fixed same-module calls to compact `effect fn` declarations by connecting
  their inferred environment, failure, and success contracts to canonical
  callable resolution.
- Added regression coverage for normal, `$`, grouped, and generic compact
  effect applications while preserving explicit and imported effect paths.

## [0.34.1] - 2026-08-29

- Fixed package execution so process, filesystem, and child-process working
  directories use the application root instead of the temporary TypeScript
  staging directory.
- Added actual CLI coverage for `.`, `./relative-package`, and absolute
  package paths while preserving relative child executable arguments.

## [0.34.0] - 2026-08-28

- Added immutable proleptic Gregorian date, nanosecond wall-clock time,
  explicit UTC offset, offset date-time, and strict extended ISO parsing and
  formatting surfaces under `std/time`.
- Added pinned IANA `2025b` time-zone Providers for Bun and browser targets,
  including canonical zone identifiers, immutable rule snapshots, and
  explicit unique, ambiguous, or nonexistent local-time resolution.
- Added runtime ABI, Provider Contract and manifest, compile, CLI execution,
  DST transition, database-version mismatch, and Lesson 24 coverage.

## [0.33.0] - 2026-08-28

- Added deterministic `std/random` services backed by xoshiro256**, including
  reproducible fixed seeds, unbiased bounded integers, choice, shuffling, and
  pseudorandom bytes across process and browser targets.
- Added `std/entropy` as a separate host-CSPRNG service with bounded secure byte
  requests and typed unavailable or read-failure results.
- Added provider contracts and manifests, runtime ABI and standard-library
  artifacts, compile coverage, a repeatable CLI seed fixture, and executable
  provider probes for range, permutation, byte, and failure behavior.

## [0.32.0] - 2026-08-28

- Added the process-only `std/child-process` application surface with immutable
  cold commands, validated arguments, environment and working-directory
  configuration, bounded captured execution, inherited execution, and
  demand-driven streaming stdin, stdout, stderr, and exit status.
- Added shared Bun and Node child-process Providers with portable signals,
  graceful termination followed by forced kill, deterministic reaping, and
  cleanup on cancellation or provider shutdown.
- Added Reference, Analysis, runtime ABI, provider contract and manifest,
  source-level CLI fixture, and actual Bun/Node execution coverage.

## [0.31.1] - 2026-08-28

- Refreshed every provider-backed executable fixture lock after the 0.31
  toolchain version bump so the canonical release gate uses one synchronized
  runtime package version.

## [0.31.0] - 2026-08-28

- Added the process-only `std/process` application surface for arguments,
  environment lookup, portable current-directory Paths, and typed process
  signals without exposing the host process object.
- Added cancel and forward shutdown policies with root Effect cancellation,
  configurable grace periods, preserved signal exit status, and deterministic
  signal-listener cleanup.
- Promoted `std/non-empty-list` for non-empty signal subscriptions and added
  Reference, Analysis, ABI, conformance, CLI execution, and shutdown fixture
  coverage.

## [0.30.2] - 2026-08-27

- Fixed `effect fn` tail recursion so pure-function TCO does not wrap an
  Effect continuation in a synchronous loop, allowing queue workers to process
  every queued action through the runtime continuation.
- Added lowering, CLI execution, fixture, specification, and review coverage
  for effect-aware tail recursion.

## [0.30.1] - 2026-08-27

- Added executable parity coverage for the standard `Functor`, `Applicative`,
  and `Monad` operators across Maybe, Either, Array, List, Effect, Task,
  Stream, and Signal, including the expected negative Signal `Monad`
  diagnostic.
- Added the canonical operator-parity sample, Playground/WASM integration,
  and stage-by-stage review documentation for named methods and operators.

## [0.30.0] - 2026-08-27

- Completed the ordinary `std/console`, `std/log`, and process-only
  `std/stdin` application surfaces with canonical Console, Logger, and Stdin
  service requirements.
- Added ordered structured logging, compact `Show` console rendering, and
  captured or live host adapters without treating Logger as a Console alias.
- Added bounded chunk and strict UTF-8 line input with sticky EOF, empty-line
  distinction, concurrent-read rejection, cancellation-safe buffering, and a
  cold non-replaying line Stream, plus executable CLI and browser coverage.

## [0.29.1] - 2026-08-27

- Fixed browser WebSocket provider packaging so staged browser builds
  resolve the shared host provider through `@seseragi/runtime`.
- Added runtime package staging regression coverage for the browser
  WebSocket provider import boundary.

## [0.29.0] - 2026-08-27

- Added the opaque portable `std/path` lexical model for POSIX, drive, UNC,
  and relative paths with validated parsing, normalization, composition, and
  component queries.
- Connected the normative `std/fs` application surface to the shared Bun and
  Node Filesystem Providers, including Bytes and UTF-8 I/O, metadata,
  directories, atomic writes, streaming, and typed path-aware failures.
- Added owner-checked file, directory, and temporary handles with cleanup
  across success, typed failure, cancellation, and provider shutdown, plus
  executable Lesson 25 and portability/conformance coverage.

## [0.28.0] - 2026-08-26

- Unified the official SQLite and PostgreSQL `Decoder` APIs under their
  `Functor` and `Applicative` instances, including curried constructors with
  three or more columns through `<$>` and `<*>`.
- Removed the public and generated-runtime `map2` decoder path from both
  database packages; applications should migrate decoder composition to the
  standard Applicative operators.
- Added source-level and fake-driver regression coverage that asserts decoded
  field values for both databases and preserves left-to-right failure
  semantics.

## [0.27.0] - 2026-08-26

- Added opaque browser `File` and `Blob` values with metadata, bounded whole
  reads, and pull-based chunk streaming that preserves cancellation cleanup.
- Added portable streaming `multipart/form-data` construction with
  library-owned boundaries, explicit MIME types, and text, Bytes, or Body
  parts without exposing host `FormData` values.
- Connected ordinary Seseragi source from file selection through metadata and
  an HTTP/2 multipart upload, with target-aware import diagnostics and an
  explicit unknown HTTP-version result for browser Fetch responses.

## [0.26.0] - 2026-08-25

- Replaced the official SQLite decoder `map2` surface with `Functor` and
  `Applicative` composition, so curried row constructors use `<$>` and `<*>`.
- Preserved canonical nominal identity for qualified namespace imports during
  generic type hydration, fixing imported Applicative dispatch for types such as
  `sqlite.Decoder<A>`.
- Aligned the TypeScript SQLite runtime with the opaque Decoder newtype while
  retaining compatibility with decoder artifacts generated before this change.

## [0.25.0] - 2026-08-25

- Added the portable `std/sse` event model, UTF-8 encoder and bounded parser
  for multiline data, event names, IDs, retry metadata, comments, and explicit
  `Last-Event-ID` requests.
- Added streaming HTTP server responses with provider backpressure and request
  scope ownership through final write, disconnect, cancellation, and cleanup.
- Connected ordinary Seseragi source through a Bun SSE server-to-client E2E,
  keeping transport failure, parse failure, remote end, and cancellation
  distinct while leaving reconnect and JSON handling to application policy.

## [0.24.0] - 2026-08-25

- Added portable `std/websocket` client connections for browser, Bun, and Node,
  with ordered text and Bytes events, subprotocol selection, explicit close
  events, and scoped cancellation.
- Added the process-only `std/websocket/server` application contract and Bun
  and Node providers without exposing host upgrade APIs to portable code.
- Bounded receive and pending-send queues with typed overflow and backpressure
  failures, and fixed browser-to-process E2E behavior across both server hosts.

## [0.23.0] - 2026-08-25

- Added the cold `std/http.exchange` request/response Stream with explicit
  response-head, body-chunk, and trailer events plus streaming request bodies.
- Added the Provider subscription operation and demand bridge, preserving
  bounded pull, typed failures, early termination, cancellation, and exactly-once
  resource cleanup without exposing Provider-specific Stream values.
- Connected Bun and Node streaming HTTP transports, documented the browser
  Fetch version boundary, and added executable backpressure and trailer fixtures.

## [0.22.0] - 2026-08-24

- Added the portable `std/stream` cold source, sequential transform, merge,
  validated capacity, lossless bounded buffer, and scoped terminal surfaces.
- Preserved exact downstream pull demand, deterministic same-turn merge
  failure selection, and producer cleanup across early stop, typed failure,
  normal completion, and cancellation.
- Added standard Stream Functor, Applicative, and Monad dictionaries, a shared
  Provider pull bridge, and executable Lesson 17 CLI and Playground fixtures.

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
