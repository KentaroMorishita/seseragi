# Change Log

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
