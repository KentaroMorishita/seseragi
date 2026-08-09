# Change Log

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
