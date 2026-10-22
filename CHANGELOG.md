# Change Log

## [0.61.81] - 2026-10-23

### Added

- Rebuild selected pure program-WASM modules and the generated TypeScript UI through standard package dev/watch, including external path dependencies. Keep the last complete site and reload version after failed edits. Cancel the UI and finish its finalizers before closing remaining program owners and callbacks for full-page reload; also drain late instantiation. Preserve explicit disposal and trap retirement. UI state-preserving HMR, program-WASM source maps and the general Effect ABI remain unsupported.

### Fixed

- Pin each development reload client to the build version served with its HTML, so a newer version observed during startup still triggers an update.

## [0.61.80] - 2026-10-23

### Fixed

- Open shared Playground source in a separate saved workspace with empty stdin; preserve the preceding local workspace and restore edits to the shared copy after reload. Scope application Storage to each workspace or Tour lesson and persist its identity before Storage execution. Legacy shared application data is retained but is no longer exposed to preview programs.
- Block preview HTTP/WebSocket and external images until Tools → Network is enabled for the current code; reset permission when code changes and avoid loading HTML images while Text is selected. Keep popup and parent-navigation permissions disabled.
- Run non-DOM programs in a dedicated Worker with a 10-second deadline, restart unresponsive compiler Workers, and bound output and channel messages. Add cooperative computation guards for generated DOM code and refuse arbitrary foreign JavaScript with DOM in Preview; use the CLI only for projects you trust. These controls do not provide a strict browser heap quota or interrupt every native built-in operation.

## [0.61.78] - 2026-10-22

### Added

- Build explicitly selected pure program-WASM modules and a Seseragi TypeScript UI from one Web package. Generate typed loader imports, preserve package ownership and initialization, and include WASM assets and receipts in the normal artifact manifest. Publish the complete site together; keep the preceding artifact on compilation or bundling failure. Program-WASM watch/HMR and Effect ABI remain unsupported.

## [0.61.77] - 2026-10-22

### Fixed

- Update the CI Node.js 24 LTS runtime to 24.22.1 for the October 21, 2026 security fixes. Generated JavaScript deployments must update their own Node runtime separately.
- Validate dynamic scalar URL attributes and custom form action URLs using the same WebUrl policy as typed HTML props. Reject unsafe initial values and updates before DOM writes, preserve typed failure/cleanup behavior, and reject raw HTML via srcdoc.
- Evaluate Playground and Tour programs, foreign JavaScript, runtime services and interactive DOM inside an opaque sandbox iframe. Keep application Storage behind its namespaced, budgeted operation channel and Navigation inside private preview history.
- Validate the HTTP Host before serving development HTML, assets, source maps, reload metadata or build-failure pages. Read complete headers within fixed size/time limits and reject missing, duplicate or unrelated authorities while preserving explicit hosts, IPv4/IPv6 and loopback localhost.
- Update vulnerable Playground build dependencies and extension runtime/packaging dependencies within their existing version ranges. One upstream braces advisory remains in packaging tools because no fixed release is available.

## [0.61.76] - 2026-10-22

### Added

- `experimental build-wasm`で単相pure関数をprogram WASMへ生成する。checkedな公開型からUnit／Bool／Int／String、Tuple、標準Maybe／Eitherと返却関数のcodec、ES module loader・型宣言・build metadataを出力する。
- 返却callbackのstrong rootと明示detach、instance dispose、trap後のretireと再生成を扱う。source引数と返り値関数の適用段階を保ち、通常disposeはmanaged root0を検査する。型付きのdomain failureとEffect failureは分け、Effect／Ref／providerのWASM ABIは未対応。
- 別のSeseragi sourceから生成したTS UIで、呼出し・Either・callback解放・instance解放と再生成を試すsampleを追加する。assetの配置は明示的なsample assemblyで、標準build/watchへの統合は残る。
- checkedな完全関数型を使い、表現できる閉じた型のpure自己参照関数へTSの型注釈を出す。再帰の型推論によるTS7023を避け、generic・辞書・未知のnominal型へ型を補わない。

## [0.61.75] - 2026-10-22

### Added

- Execute native readTextUtf8 and writeTextUtf8 Effects on supervised file workers while other Fibers continue. Keep runtime values on the runtime thread, close owned files and join each worker before resuming the Effect. Cancellation waits for an in-flight OS call to return and checks between chunks; forced OS abort and external shutdown handling remain unsupported.

### Fixed

- Wait for masked lexical finalizers, late filesystem acquisition and pending handle operations before publishing TypeScript Fiber cancellation. Preserve cleanup defects and close owned handles after unavailable host operations drain.

## [0.61.74] - 2026-10-21

### Added

- Run structured Fiber and Deferred programs with the experimental native target. Suspend and resume cold Effects on a cooperative FIFO scheduler, detach cancelled waiters, and wait for child cleanup at scope exit. Preserve atomic resource registration, masked finalizers and traced continuation roots. Actual asynchronous host I/O and external shutdown handling remain unsupported.

### Fixed

- Preserve checked Deferred allocation payloads as TypeScript type arguments and accept source Unit arguments in Deferred.make and Effect.yieldNow runtime declarations. Generated programs pass strict checks without adding casts.

## [0.61.73] - 2026-10-21

### Added

- Execute UTF-8 file transformations with the experimental native target: portable Path operations, cold readTextUtf8 / writeTextUtf8 Effects, Replace / CreateNew / Append, typed filesystem and UTF-8 failures, and fromEither. Close owned file handles after success or failure and preserve primary I/O errors. Source files and linked path packages use the same checked contracts; explicit JavaScript filesystem provider selections are rejected before output. Async I/O, cancellation and the remaining filesystem API are still unsupported.

### Fixed

- Load selected provider exports in bundled process releases through compiler-generated data properties, preserving the provider loader's validation and accessor rejection. Accept Path.current's source Unit argument in its TypeScript runtime signature so generated calls pass strict type checking.

## [0.61.72] - 2026-10-21

### Fixed

- Retain checked parameter and result identities inside function values, including Show aliases and anonymous Unit callbacks in a function whose generic binder is named Unit. Preserve scoped binders, canonical owners and newly quantified let dictionaries through inference and substitution, keeping source application stages and returned functions distinct. Core and runtime ABIs remain unchanged; unproved callable schemes remain tracked in #887.

## [0.61.71] - 2026-10-21

### Fixed

- Preserve the canonical Prelude Unit source parameter of functions with omitted parameters when their own generic binder is named Unit. Keep generic returned functions and explicit source parameters distinct, including saved aliases and package interfaces. Retain source evaluation stages and dictionary scopes without changing the Core schema or runtime ABI. Remaining unproved Unit schemes stay tracked in #887.

## [0.61.70] - 2026-10-21

### Fixed

- Preserve the checked Prelude Unit identity when passing saved Unit values to trait methods whose own type parameter is named Unit. Reconcile canonical declaration types with the caller's resolved intrinsic representation, keeping lexical type parameters and same-named nominals distinct. Preserve source stages, returned functions, dictionary frames, Ref cycles, cold Effects and finalizers across local and package calls. Remaining Unit schemes stay tracked in #887.

## [0.61.69] - 2026-10-20

### Fixed

- Normalize an omitted trait method parameter to one anonymous Prelude Unit stage in checked call signatures and method declarations. Preserve returned functions, dictionary-only factories, actual provider source stages and scoped canonical Unit identity across saved values, cold Effects and resource lifetimes. Retain Unit provenance through inference, interface projection and TypeScript import planning without weakening native evidence guards. Preserve the checked Unit success type when `succeed` elides its literal argument, including imports into a scope with a nominal Unit. Remaining unproved application and dictionary schemes stay tracked in #887.

## [0.61.68] - 2026-10-20

### Fixed

- Include the anonymous Prelude Unit parameter when checking an instance method with omitted source parameters against its trait contract. Reject extra-Unit provider signatures before Core or TypeScript generation, including unused instances and imported traits. Preserve explicit Unit providers, canonical Unit identity, alpha-renamed higher-kinded binders, method constraints, and valid single-argument implementations returning functions. Remaining method application schemes stay tracked in #887.

## [0.61.67] - 2026-10-20

### Fixed

- Preserve the actual method stages when directly applying a checked source method with method-owned `where` constraints through a primary parameter dictionary. Obtain the validated method value before evaluating source arguments, keeping one-parameter implementations returning a function distinct from two-parameter implementations. Reuse canonical owner, kind and evidence-frame checks; retain operator source order, saved partials, returned closures, Ref cycles, cold Effects and finalizers. Unproved schemes and implicit Unit provider signature checks remain tracked in #887.

## [0.61.66] - 2026-10-20

### Fixed

- Preserve selected method stages when a saved source method has its own `where` constraints. Specialize primary and child dictionaries before exposing source arguments, using a private dictionary value entry while retaining the existing call entry. Check source declaration owners, binder kinds, complete method types and evidence frames independently in native generation. Keep `Never` separate from same-named nominal types and binders, and match constructor parameters by their recorded scope identity. Preserve captured and generalized aliases, returned closures, Ref cycles, cold Effects and finalizers. Remaining method application schemes stay tracked in #887.

## [0.61.65] - 2026-10-20

### Fixed

- Preserve the selected first method stage when a checked source method declares constructor-kind type parameters. Keep declaration binder kinds, captured and generalized dictionaries, aliases, fixed prefixes and returned functions distinct from source argument stages. Independently validate native method value kinds and constructor bindings, including child dictionaries, Ref cycles, cold Effects and finalizers. This covers declarations with source parameters and no method-owned constraints; remaining schemes stay tracked in #887.

## [0.61.64] - 2026-10-20

### Fixed

- Preserve selected method stages for checked source traits with constructor-kind parameters, value-kind method binders and no method-owned constraints. Capture available dictionaries or introduce generalized dictionary-only factories, including module and local alias chains, without delaying a supplied source argument. Validate canonical declaration owners, constructor slots and fixed prefixes, method binders and evidence frames independently in native generation. Keep one-parameter implementations returning a function distinct from two-parameter bodies, and preserve child dictionaries, Ref cycles, cold Effects and finalizers. Unproved method schemes remain tracked in #887.

## [0.61.63] - 2026-10-20

### Fixed

- Preserve canonical Ord `compare` values in captured and generalized lets and constrained module aliases. Apply the selected method's first source stage when creating a partial, while keeping a two-parameter body and an ordinary Unit function deferred until their source arguments are supplied. Validate the Ordering result, value-kind binders, provider identity and dictionary frame independently in the native backend; preserve saved closures, child dictionaries, Ref cycles and resource finalizers. Remaining method schemes stay tracked in #887.

## [0.61.62] - 2026-10-19

### Fixed

- Reject type parameters that shadow an enclosing declaration's parameter, as required by spec 2.11. Preserve sibling scopes and ordinary nominal/value shadowing; report unused invalid trait methods and nested functions before code generation.
- Retain checked source trait method declarations in shared Typed HIR and CoreIr. Use their canonical owner, ordered binders, kinds and complete signature to preserve selected method values from abstract primary dictionaries, including captured and generalized lets, constrained module aliases, saved partials, returned functions and child dictionaries. Validate native declaration membership and dictionary frames independently of call-site metadata; preserve resource lifetimes and the separate standard method ABI proofs. This slice covers value-kinded declaration parameters without method-owned constraints; unproved scopes and remaining method staging stay tracked in #887.

## [0.61.61] - 2026-10-19

### Fixed

- Preserve the actual selected method value in generalized Prelude Eq and arithmetic aliases. Introduce their dictionaries before source application, keep fresh let binders separate from enclosing parameters, and run the selected first stage when its first source argument is supplied. Preserve constrained module aliases and function-valued results using checked native substitutions and instantiated TS signatures. Validate factory frames, types, kinds, child evidence and stored closures; retain unproved signatures and broader method-parameter staging in #887.

## [0.61.60] - 2026-10-19

### Fixed

- Evaluate trait operator operands once in source order before passing their saved values in the selected method's ABI order. Preserve staged Functor/Applicative/Monad references, supplied partial arguments, method-owned dictionaries and returned functions in shared Core lowering. Check lexical and imported operator signatures by canonical owner, kind and scoped binder identity; keep unproved operator references rejected. Generalized method-reference staging remains tracked in #887.

## [0.61.59] - 2026-10-19

### Fixed

- Capture already available source dictionaries in monomorphic method aliases, so applying a saved Eq or arithmetic method runs its first source stage immediately. Preserve lexical binder and owner identity, generalized let dictionary introduction, selected child dictionaries and closure lifetimes; validate parameter method signatures in the native ABI. General method-parameter staging and source-order operator permutations remain tracked in #887.

## [0.61.58] - 2026-10-19

### Fixed

- Preserve the selected method value for closed monomorphic references, so applying a saved `add` or `eq` runs its first source stage immediately. Keep generalized dictionary introduction separate, materialize the required TypeScript dictionary imports, and verify native method owners, types, constructor kinds and returned-function stages. Generalized method-reference staging and source-order operator permutations remain tracked in #887.

## [0.61.57] - 2026-10-19

### Fixed

- Preserve the selected Eq method stage when saving equality and negated equality operator references. Run a native user Eq method’s first stage before evaluating the right operand, and retain checked Int Eq dictionaries, generic child evidence, saved closures, cold Effects and finalizers. Shared method-reference staging remains tracked in #887.

## [0.61.56] - 2026-10-19

### Fixed

- Evaluate supplied primitive trait-method arguments when a partial is created, and preserve the first stage of selected arithmetic operator references. Keep source argument types and selected dictionaries separate from returned functions. Support checked native Int Add/Sub/Mul and String Add dictionaries. Other operator-reference paths remain tracked in #887.

## [0.61.55] - 2026-10-19

### Changed

- Use Bun 1.4.3 for development, CI and release verification, with one `.bun-version` file shared by every workflow. Preserve locked dependency versions and the existing TypeScript checker. Require owned browser verification daemons and their child processes to exit after each run (#891).

### Fixed

- Abort cancelled Bun HTTP requests without returning a status-zero response or publishing an HTTP success. Preserve request cancellation, server shutdown and exactly-once resource finalizers on Bun 1.4.3 (#891).

## [0.61.54] - 2026-10-18

### Fixed

- Evaluate supplied comparison and uncurried standard-library partial arguments once at creation. Preserve the selected compare method stage and Ordering-to-Bool conversion, and support certified native Int Ord dictionaries and canonical Ord instances with their proved Eq parent. Other specialized operator ABI paths remain tracked in #887.

## [0.61.53] - 2026-10-18

### Fixed

- Evaluate supplied source arguments once when creating constrained partial applications of source-defined functions and methods. TypeScript retains the already applied prefix, and native local functions and constrained methods preserve the same strict evaluation, source arity, selected evidence, Ref captures, cold Effects and resource finalizers. Admit proved Maybe display partials through these paths; specialized comparison and standard-library staging remains tracked in #887.

## [0.61.52] - 2026-10-18

### Added

- Execute safe Array indexing in the Rust target using the checked canonical Array, Int and Maybe types. Evaluate the receiver and index once in source order, return Nothing outside the array, and retain selected functions, dictionaries, Refs and cold Effects after the container is dropped (#883).
- Execute standard Maybe Show and Debug with the selected child dictionary. Preserve nested display depth, skip the child for Nothing, and keep Defect and finalizer behavior (#883).

## [0.61.51] - 2026-10-18

### Fixed

- Preserve checked higher-kinded callable signatures when generating TypeScript for unconstrained calls and saved function references. Keep instantiated return types through imported functions, constructor prefixes, partial applications and returned closures without changing source arity, evidence or evaluation order (#875).

## [0.61.50] - 2026-10-18

### Added

- Execute instance methods with their own type-class constraints in the Rust target. Keep instance factory evidence separate from method evidence, validate instantiated canonical types and source arity, and preserve saved methods, partial calls, returned closures, imported providers, Ref captures and cold Effects (#872).

### Fixed

- Carry the checked method contract through Typed HIR and CoreIr so TypeScript passes method dictionaries after source parameters and before arguments to a returned function. Preserve deferred partial-call expressions, evaluation order, Defects and resource finalizers (#872).

## [0.61.49] - 2026-10-18

### Fixed

- Distinguish generalized let lambdas from named local function declarations in shared CoreIr. Keep a shadowing lambda's references in the preceding lexical scope on TypeScript, and reject unsupported native named-local recursion before it can resolve to an outer value. Keep calls through same-named parameters, local bindings and pattern bindings out of the TypeScript self-tail trampoline (#879).

### Added

- Execute nonrecursive constrained local functions and generalized lambdas in the Rust target with explicit dictionary stages. Preserve source arity, inherited evidence frames, canonical trait owners and checked constructor kinds across saved aliases, partial calls, returned closures and path packages. Keep Ref captures traceable, cold Effects and finalizers ordered, and enclosing self-tail calls stack safe. Reject recursive locals and unproved evidence or binder/staging shapes before output (#876).

## [0.61.48] - 2026-10-17

### Fixed

- Keep identity-checked generic substitutions when an imported alias return uses a different display name from its parameter. Preserve scoped binders and canonical child dictionary evidence through renamed, namespace and facade imports, including saved closures and rank-1 local lambdas. Continue rejecting distinct nominal owners, conflicting arguments and opaque representation access before generation (#873).

## [0.61.47] - 2026-10-17

### Added

- Use transparent wrapper aliases as trait instance constructors when a checked eta proof preserves every first-order parameter in order. Share the canonical owner across registration, method contracts, selected dictionaries and overlap checks, including fixed prefixes and imported alias chains. Preserve fully supplied alias value arguments in method contracts and diagnose unproved signatures even on unused instances. Keep unsupported reordered, repeated, erased and higher-kinded alias templates positioned and rejected before generation; general alias constructor support remains in progress (#854).

## [0.61.46] - 2026-10-17

### Fixed

- Distinguish proved instance overlap, proved disjointness and unsupported higher-kinded comparisons. Normalize constructor section slots for eta-equivalent heads such as `Box` and `Box<_>`, preserve canonical owners and standard functional dependencies across imported providers, and reject unproved combinations before generation with a positioned diagnostic. Keep single HKT instances and provably disjoint heads valid. General alias constructors and higher-order coherence remain in progress (#854).

## [0.61.45] - 2026-10-17

### Fixed

- Reject overlapping generic and concrete trait instances before code generation, including derived instances, imported dictionaries and standard functional dependencies. Compare canonical owners and scoped type parameters without changing selected evidence, source arity or evaluation order. Keep disjoint heads and repeated imports of the same provider valid. Higher-kinded constructor sections and unapplied alias constructors remain in progress (#854); generic impl parameter mapping is tracked in #867.

## [0.61.44] - 2026-10-17

### Fixed

- Normalize represented instance-head aliases by resolved type owner for registration, method contracts, dictionary selection and duplicate checks. Keep canonical nominal owners through renamed facade exports, and reuse checked source type imports when generating TypeScript. Reject orphan and erased or unrepresented instance heads before output. General generic/concrete overlap checking and unapplied alias constructors remain in progress (#854, #865).
- Make interface-only conformance artifact refresh resolve standard modules through the same driver path as the canonical checker.
- Validate type arguments and child evidence for native Show/Debug factories on ordinary calls as well as collection display. Preserve the selected language methods, source arity and resource lifetimes.

## [0.61.43] - 2026-10-17

### Fixed

- Keep top-level declarations and public visibility when a deprecated clause
  shares a line with the declaration or crosses comments and newlines. Share
  modifier boundaries across CST, surface parsing and validation, retain
  message/version/source metadata through inferred Effect exports and
  reexports, and diagnose invalid clauses before output or execution (#856).

- Reexport a merged type and its same-name constructor with one TypeScript
  specifier when the canonical binding and alias match. Retain both source
  namespaces and module initialization edges in the IR (#862).

## [0.61.42] - 2026-10-17

### Added

- Execute native Array Show and Debug with the selected element dictionary,
  including user, generic and higher-kinded instances, saved closures and
  closed partial calls across modules and path packages. Preserve element
  order, the shared display depth bound and Ref/Effect resource lifetimes.

### Fixed

- Match local instance heads using resolved nominal types and retain inferred
  Array evidence under type and parameter shadows, so a user type named Array
  cannot replace the standard literal-array display dictionary.
- Join DOM fixture worker exits independently from stderr collection. Preserve
  diagnostics in an owned file and bound shutdown, forced exit and startup
  cleanup, including an inherited stderr handle (#860).

## [0.61.41] - 2026-10-17

### Fixed

- Diagnose missing parameter or result annotations inside trait and instance
  methods before recovery can discard them. Validate canonical Prelude instance
  methods against their registered signatures, including required methods,
  higher-kinded parameters and nominal result types, so incomplete dictionaries
  cannot reach TypeScript or native code generation (#851).
- Run the VS Code product-test SDK with Node.js and a fresh private download
  cache. Preserve archive validation, the installed CLI/VSIX journey and failure
  receipts while avoiding Bun child-process pipe truncation (#824).

## [0.61.40] - 2026-10-16

### Added

- Execute the selected tuple Show and Debug dictionaries in native programs,
  including custom, generic and higher-kinded element instances, saved closures
  and closed partial calls. Preserve child evaluation order, the shared display
  depth bound and Ref/Effect resource lifetimes across modules and path packages.

### Fixed

- Respect an ADT's type parameter scope when generating TypeScript payloads
  and constructors, so higher-kinded parameters use the existing erased type
  representation and value parameters keep their own identity.

## [0.61.39] - 2026-10-16

### Fixed

- Emit one TypeScript import binding when several direct or reexport paths reach the same canonical value, method or operator. Preserve each module's initialization edge so private facade initializers still run and report Defects before the entry. Verify process builds on Bun and Node in both profiles (#847).

## [0.61.38] - 2026-10-16

### Improved

- Share checked scalar global-read facts between TypeScript and Rust self-tail loops. Closed monomorphic Int/Bool literal bindings can use scalar loop slots while keeping each original read, declaration and initializer in place. Preserve canonical package/module owners, lexical shadowing, source arity, selected evidence and failure order; unproved bindings keep the ordinary implementation (#843).

## [0.61.37] - 2026-10-16

### Added

- Initialize monomorphic native global values, structures, closures and cold synchronous Effect plans once in checked dependency / source order. Keep canonical imports, selected evidence and source arity, and retain global roots through scope finalizers before releasing dictionaries and Ref cycles. Reject unproved polymorphic value shapes before output (#839).

### Fixed

- Preserve source-import initialization order independently of type checking, and keep eager initializers and unused import edges during TypeScript application pruning so Defects cannot disappear. Use checked CoreIr summaries to preserve task-only foreign modules' lazy loading when imports use only types (#844).
- Preserve canonical identity when calling monomorphic imported function values, including namespace and renamed imports, in the shared frontend for both TypeScript and Rust (#839).
- Report TypeScript process application initialization failures as runtime Defects with exit 70 and text / JSON diagnostics, before entry or provider setup. Keep seed initialization order and literal dynamic imports for normal and bundled execution (#841).

## [0.61.36] - 2026-10-16

### Added

- Link checked multi-module projects and path dependency packages through the experimental Rust target without TypeScript output planning (#837). Preserve canonical symbols, imported dictionary owners, generic / HKT constraints, returned function stages and synchronous Effect / Ref lifetimes. Validate the existing project lock and reject private / opaque access and unsupported imported code before writing outputs. Top-level initialization, derived dictionaries, supertraits and remaining standard APIs continue separately under #812.

## [0.61.35] - 2026-10-16

### Added

- Execute persistent List literals, cons function values, checked prefix/rest patterns, selected Show / Debug dictionaries and bounded list operations through the experimental Rust target (#834). Share tails without copying, preserve Float / Char / String display and callback order, and trace List edges through Ref cycles and synchronous Effect scopes. Verify deep construction, traversal and destruction on a small native stack. Collection typeclass instances, comprehension and remaining list APIs remain outside this slice.

## [0.61.34] - 2026-10-16

### Added

- Execute binary64 Float values, selected arithmetic / Show / Debug / Zero / One evidence, deterministic comparison and classification APIs through the experimental Rust target. Preserve NaN, infinity, negative zero, reusable partials, and Float payloads in Ref / synchronous Effect scopes. Match canonical TypeScript display across decimal midpoints and exponent boundaries without a JavaScript runtime (#832). Power, parsing and rounding conversions remain outside this slice.

## [0.61.33] - 2026-10-16

### Fixed

- Accept the specified binary, octal and hexadecimal Int literal spellings through the shared frontend. Diagnose complete malformed numeric tokens and safe-range overflow, normalize equivalent signed patterns and typed integer values for both TypeScript and Rust, and preserve source spelling in the formatter (#826).

## [0.61.32] - 2026-10-16

### Added

- Execute Unicode scalar Char values through the experimental Rust target, including standard conversions, checked patterns, Eq / Ord / Show / Debug evidence, reusable closed Char partials and synchronous Ref / Effect payloads (#825). Verify the full scalar domain and compare generated TypeScript on Bun and Node.

### Fixed

- Type the web-file pending byte buffer to accept stream chunks in strict TypeScript consumers without changing emitted runtime behavior (#827).

## [0.61.31] - 2026-10-15

### Added

- Recognize checked scalar self-tail loops through shared typed CoreIr analysis. Generate TypeScript statement loops without per-step markers or argument arrays, and specialize private native helpers to Int / Bool while preserving curried calls, reusable partial applications, selected standard evidence and overflow evaluation order (#822). Record cross-backend boundary checks and measured results for the supported scalar kernel.

## [0.61.30] - 2026-10-15

### Added

- Add an experimental Rust backend from checked CoreIr, with `experimental emit-rust` and `experimental build-native` commands for standalone source generation and Bun-free native execution (#808). The synchronous slice supports cold Effects, typed failure/recovery, Ref, Console and resource scopes with cycle collection; async providers, package linking, foreign ABI and standard Wasm program output remain outside this slice (#810).
- Share source-arity, closed local evidence and free-variable analysis between backend implementations; use direct saturated calls, lazy dictionary sharing, explicit closure environments and iterative value destruction in the Rust target (#808). TypeScript performance probes and native ownership fixtures record the implementation boundaries without claiming full-language or cross-backend speed guarantees.
- Compile direct pure self tail calls to loops while preserving per-iteration closure snapshots, reusable partial applications, lexical shadowing and changing generic dictionaries (#811).

### Fixed

- Execute deeply deferred Effects through the TypeScript runtime's instruction loop, preserving coldness, replay and failure identity without relying on JavaScript engine tail-call support. Verify 100,000 deferred steps on Bun and Node (#808).
- Select the standard Show dictionary for canonical external ConsoleError and StdinError main contracts without accepting unrelated same-name user types (#819).
- Evaluate shadowed non-recursive let initializers in their outer scope in generated TypeScript, including closure captures and polymorphic bindings in pure, Effect and monad sequences (#820).
- Retain edits and manifest repairs made during a dev build as pending changes, so graph refresh cannot consume them before a successful rebuild (#821).

## [0.61.29] - 2026-10-13

### Fixed

- Prove match coverage for recursive types and large product domains without eagerly enumerating values, retain nested collection/product identity, and report bounded analysis limits instead of accepting unproved matches (#793).
- Accept signed integer and nested rest patterns, reject malformed patterns before compilation, and distinguish nominal `Bool` values from primitive Boolean predicates (#794, #803).
- Preserve primitive and nominal type identities through trait evidence selection so a same-name user type cannot replace a standard dictionary (#804).
- Check semantic argument identity even when display types match, rejecting primitive/nominal argument confusion before execution (#805).
- Preserve nominal primitive-spelling types in generated TypeScript annotations and constructors instead of rendering them as builtin scalar types (#806).
- Keep reusable tail-recursive partial functions and captured iteration closures independent while preserving constant-stack execution (#795).
- Revalidate generated semantic adapters against current policy and input evidence, and prevent declaration conversion from overwriting authored inputs (#796, #802).
- Resolve nested npm dependencies from their actual importer ancestry consistently with virtual browser resolution and Node ESM (#797).
- Cancel pending Playground and Tour preview preparation completely, restore Run after cancellation, and allow an explicit retry after transient WASM initialization failure (#798, #799).
- Serialize official extension LSP restarts, stop pending or rejected startup safely, retain the legacy migration gate, and accept workspace child names beginning with two dots (#800, #801).

## [0.61.28] - 2026-10-13

### Added

- Warn when every resolved use of a public `MutableSignal<A>` parameter is a direct canonical read, sharing the same evidence between CLI and LSP (#611). The warning suggests reviewing `Signal<A>` for the input contract and excludes writes, aliases, forwarding and closure captures; it does not automatically change public signatures or claim deep immutability.

### Fixed

- Keep the Playground editor height rule off its tooltip container, preventing an extra viewport of page height and unstable preview scroll during full-page browser captures.

## [0.61.27] - 2026-10-12

### Added

- Route declaration conversion through a portable, source-independent semantic graph with lexical generic relations, recursive identities, overloads, namespace ownership, strict schema validation and source provenance (#475, #476, #477).
- Preserve TypeScript constraints, projections, class identity and overload ownership, with bounded normalization and explicit reasons for unresolved meanings (#478–481).
- Generate separate checked JSON snapshot, nullable and tagged-union APIs from fresh explicit policies, without treating opaque or mutable host values as records (#482–484).
- Persist runtime artifact and ABI evidence, validate browser/Node/Bun package selection, and execute a pinned published npm package through normal generated imports in CLI and Playground (#485–487, #490).
- Report interpreted and projected compatibility separately; verify semantic and pinned npm corpora and show navigable meaning, policies and provenance in Inspector (#488, #489, #491).
- Reuse the common graph and projector from a bounded JSON Schema frontend with no TypeScript frontend in its production dependencies (#492).
- Share evidence-backed lint diagnostics between the official CLI and LSP for unused imports/private declarations, discarded effects/subscriptions and unreachable literal-Boolean branches (#611).

### Fixed

- Reject malformed expression bodies before compilation while retaining useful linked editor queries and original parser diagnostics during incomplete edits (#783).
- Preserve the returned-function boundary of JSON decoder factories (#784).
- Emit persistent process runtime and generated modules as executable JavaScript for Node and Bun, preserving lazy imports, composed source maps and truthful artifact measurements (#785).
- Retain generic foreign binders and opaque type arguments, selecting checked primitive codecs for concrete calls and rejecting unresolved forwarding (#786).
- Preserve `__proto__` as an ordinary record field, including generated checked JSON APIs (#787).
- Stage the declared relative host import closure for standalone run and process builds, preserving lazy failures, host package scope and relocatable artifacts (#788).
- Retain preferred Playground pane sizes across temporary narrow or short viewports, while keeping explicit resize controls and persistence clamped to the current layout (#790).
- Reject modified generated bindings and imports of retained outputs whose entry or projection policy has been removed (#476).
- Produce canonical Linux WASM before updating PR artifacts while keeping normal PR freshness verification strict.

## [0.61.26] - 2026-10-12

### Fixed

- Keep Playground and Tour editing responsive by running compiler work in a Web Worker and retaining only the latest pending live analysis.
- Defer the Playground Reference catalog's highlighted cards until it is opened, and release them when it closes.
- Give editor diagnostic popups an opaque background when they share a tooltip with type information.

## [0.61.25] - 2026-10-08

### Fixed

- Give Web UI regression samples and viewports independent browser contexts and timeout budgets while preserving all visual baselines and assertions.

## [0.61.24] - 2026-10-08

### Added

- Inspect Playground declaration inputs beside read-only generated bindings, conversion reports and navigable diagnostics; invalidate stale inspections when external inputs change (#473).

## [0.61.23] - 2026-10-08

### Added

- Import converted Playground bindings through ordinary `gen/*` modules, sharing generated identities across compilation and analysis, diagnosing collisions, and replacing or invalidating stale generated artifacts across edits and reloads (#472).

## [0.61.22] - 2026-10-07

### Added

- Edit, persist and convert TypeScript declarations, binding settings and host metadata as Playground workspace files, using the shared portable converter and rejecting stale conversion responses (#470, #471).

## [0.61.21] - 2026-10-05

### Fixed

- Apply function-valued record fields directly, including nested, generic, lexical, and partially applied calls (#741).
- Diagnose source/generated module identity collisions and imports before the generated root exists instead of panicking (#742, #743).
- Plan imported types used by foreign parameters, results, values, and nested namespaces, including generated DTS bindings (#744).

## [0.61.20] - 2026-10-02

### Fixed

- Infer and generalize non-recursive let lambdas with captured-environment and trait constraints, preserving rank-1 and monomorphic recursion rules (#683).
- Preserve constrained callable aliases and their dictionary evidence across top-level, block, Effect, and monadic bindings (#738).
- Reject unresolved polymorphic callable storage before emitting TypeScript with free type variables, and honor explicit type arguments on callable values (#739).
- Diagnose invalid or reserved declaration names, missing ordinary-function and public-let annotations, and expressions at module top level even when declarations are unused; preserve record types in instance and impl headers (#679, #709).
- Round Float HalfUp correctly at binary64 boundaries without rounding an intermediate addition, with native and WASM execution regressions (#715).

### Documentation

- Add Japanese/English examples for eleven existing byte inspection and chunk composition APIs, with checked TypeScript comparisons and explicit constructor/range explanations.
- Replace obsolete let-lambda inference and missing-annotation notes with the verified 0.61.20 behavior.
- Define source-neutral semantic interop responsibilities and failure boundaries (#474).
- Document Signal read snapshots, sequential-read interleaving, transaction publication, and measured composition costs (#610).
- Select the measured release profile for complete Docs regression builds and update browser checks for the current verified examples (#706, #740).
- Prepare the first-reader review procedure and record the requested simulated TypeScript-reader review separately from actual reader feedback (#702).

## [0.61.19] - 2026-09-30

### Fixed

- Enforce invariant collection and nominal type arguments, invariant record fields and Effect/Stream payloads while preserving explicit capability widening and direct record-width conversion.
- Preserve qualified public namespace re-exports alongside named and aliased exports, without duplicate TypeScript exports or collisions with underscore names.
- Replace the corresponding English/Japanese reference limitations with verified examples and a rejected invariant-assignment example.
- Rewrite 24 Japanese reference explanations around concrete examples and reader-facing terminology, with matching related-link labels.

## [0.61.18] - 2026-09-27

### Fixed

- #660 v0.61.17で更新漏れしていたbrowser multipart fixtureのprovider lockを同期し、canonical release gateを復旧

## [0.61.17] - 2026-09-27

### Fixed

- #658 Docsのコード文字サイズとsyntax highlightをPlaygroundの正規themeへ統一し、静的highlightでも演算子と組み込み型の分類を保持

## [0.61.16] - 2026-09-25

### Fixed

- #622 sample CLI timeoutで所有するprocess tree全体を停止し、存在しないlint pathの誤ったpackage fallback、CLI/LSP診断parityの検証漏れ、formatter並列testの一時directory衝突を修正

## [0.61.15] - 2026-09-25

### Changed

- #620 公式lintの未使用局所binding診断をLSPにも公開し、CLIと同じ解析・診断順序で編集中の更新に追従

## [0.61.14] - 2026-09-24

### Added

- #616 公式 `seseragi lint` の最小sliceを追加。linked analysisに基づく未使用局所binding診断、text/JSON出力、明示的な `--deny-warnings` に対応

## [0.61.13] - 2026-09-24

### Changed

- #614 `std/log`の明示的な操作相関と安全なエラー表示の責務を仕様化し、並行Fiber・型付き失敗を含む正規fixtureで検証

## [0.61.12] - 2026-09-23

### Changed

- #604 Web UI q0: `ElementRef`、型付きbinding target、細粒度更新、keyed region、pointer event control、DOM traceを一つの明示的なscene更新モデルとして追加

### Fixed

- release pruningが使用中のtype-only import先を保持しつつ、未使用の値importとそのモジュールを除去する

## [0.61.11] - 2026-09-13

### Fixed

- #602 formatterがネストした`match`の別armを結合せず、record field内のinline `match`を一回で収束させる

## [0.61.10] - 2026-09-12

### Changed

- #597 O03 candidate: Ishikoridome dogfoodで露出したWeb scene interaction境界（SVG / pointer lifecycle / DOM measurement）
## [0.61.9] - 2026-09-10

### Changed

- #589 O02 hotfix: verify composed source maps in installed product E2E
## [0.61.8] - 2026-09-08

### Fixed

- Preserve generic callable schemes through module, local, do-block and imported aliases, including higher-order use and generated TypeScript binders (#545).
- Accept consecutive application of returned callables while preserving declared arity and constraint evidence order across module imports (#534).
- Preserve nested HKT annotation behavior and reject forbidden outer type-parameter shadowing (#537).
- Reject empty conformance runs and multiple repository roots instead of reporting a misleading success; synchronize callable interface and Tour diagnostic fixtures (#583, #584, #585).

### Changed

- Record the decision to keep canonical `sort` and stable key-based `sortBy` for Array and List without introducing an arbitrary-comparator `sortWith` API (#369).

## [0.61.7] - 2026-09-08

### Changed

- Complete the O01 language stack: Array indexing with Maybe results, Char literals, right-associative Maybe fallback, List cons, local effect functions, local recursive groups, and opaque structs (#393, #513, #343, #298, #345, #346, #347).
- Add std/math, custom Web documents and public assets, LSP range formatting, a verified Linux glibc 2.34 baseline, development/release profiles, and the canonical benchmark runner (#406, #529, #337, #530, #342, #321).
- Resolve terminal validation findings in HKT lexical identity, standard API and Playground runtime contracts, inline lambda formatting, benchmark duration assertions, and dev asset deletion monitoring (#575, #577, #578, #579, #580, #581).
## [0.61.6] - 2026-09-07

### Changed

- #507 【#291・標準ライブラリ監査】未実装standard instanceの追跡Issueを必須化
## [0.61.5] - 2026-09-07

### Changed

- #506 【#291・標準ライブラリ】5種のMonad transformerと標準instanceを追加
## [0.61.4] - 2026-09-07

### Changed

- #505 【#291・標準ライブラリ】Sum / ProductとMonoid instanceを追加
## [0.61.3] - 2026-09-07

### Changed

- #502 【#291 ＞ #354・Effect】forEachUntil / LoopControlを標準APIへ接続
## [0.61.2] - 2026-09-07

### Changed

- #536 【#291 ＞ #357・Formatter】comment・nested block・else-ifの字下げ崩れを修正
## [0.61.1] - 2026-09-06

### Changed

- #528 【#291・Web UI】onChangeでtext / selectがchecked読取エラーになる問題を修正
## [0.61.0] - 2026-09-05

### Fixed

- Reject unpaired UTF-16 surrogates at the TypeScript foreign `String` codec
  boundary while preserving valid Unicode scalar strings byte-for-byte.

## [0.60.0] - 2026-09-05

### Fixed

- Preserve ADT match payload bindings, trait evidence, and Effect contracts in
  `do`-block match arms, including pure-to-Effect calls, failure channels, and
  imported constructors.

## [0.59.0] - 2026-09-05

### Fixed

- Erase lexically scoped higher-kinded TypeScript parameters consistently in
  generated match bodies, bindings, lambdas, and local functions while
  preserving ordinary nominal generic types.

## [0.58.0] - 2026-09-05

### Fixed

- Refine inferred inline collection elements against polymorphic call
  expectations, so Array and List literals and comprehensions retain their
  concrete element types through pure and effectful calls, `do` bindings,
  module boundaries, and generated Playground analysis.

## [0.57.0] - 2026-09-05

### Added

- Expose the existing persistent `Iterator<A>` identity through `std/iterator`,
  including lazy `unfold` and persistent `next`, across compiler, runtime,
  Reference, LSP, WASM, Playground, and conformance execution.
- Reuse the canonical `Iterable<Iterator<A>, A>` instance for Array, List,
  Range, and custom iterables without adding `Reducible<Iterator>` or a second
  iterator runtime.

## [0.56.0] - 2026-09-05

### Added

- Generate nominal `Eq`, `Ord`, and `Hash` dictionaries for derived structs,
  ADTs, and newtypes, including generic, nested, and imported evidence across
  native CLI, TypeScript, WASM, LSP, and Playground execution.
- Preserve structural equality and hash consistency, source-order ADT ordering,
  and newtype delegation; require canonical Eq evidence for Ord and Hash.

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
