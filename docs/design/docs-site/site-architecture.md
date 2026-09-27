# Seseragi Documentation Site Architecture

Status: design source for the documentation mockups. This document defines
content depth before visual design. It does not replace `docs/spec/`.

## 1. Content ownership

- `docs/spec/` is the normative source used to review correctness and drift.
- The documentation site is the complete reader-facing explanation of the
  language. A reader must not need to open the specification to understand a
  documented feature.
- Documentation pages teach one subject at a time and link to the exact
  normative section as evidence, not as a substitute for explanation.
- Public API signatures and symbol identities come from compiler metadata.
- `examples/spec/` and verified samples provide executable code. Prose must not
  invent a second, unverified version of an example.
- The website is English-first. Japanese uses the same page ids and hierarchy
  under `/ja/`; missing Japanese content falls back visibly to English.

## 2. Page-granularity rule

The specification currently contains 290 second-level normative sections.
They are not content to compress into six overview articles.

- A sidebar group is navigation, not an article.
- Every normative subject gets its own stable leaf route.
- Overview pages introduce and route; they do not absorb the leaf articles.
- Closely related pages link to each other but remain independently searchable.
- Standard Library modules get module pages, then generated pages for their
  public types, traits, functions, instances, and deprecations.
- Large contracts such as Effect, Web UI, Tooling, and Runtime Providers use
  nested sidebar groups so their leaf pages remain visible without one enormous
  flat list.

### Completeness rule

Navigation coverage alone is insufficient. Before a leaf page is considered
designed, every rule in its source specification section must be assigned to a
visible part of the article. That includes:

- accepted syntax and invalid syntax;
- static type, inference, constraints, and coercions;
- evaluation order and desugaring;
- success, typed failure, defect, and cancellation behavior;
- resource and lifetime rules;
- module visibility, package identity, and interop boundaries;
- complexity, allocation, stack-safety, and other performance guarantees;
- diagnostics and rejected configurations;
- executable examples, counterexamples, and related concepts.

The specification link is provenance. It never stands in for any of the items
above. A short summary that omits the normative behavior is a failed page.

## 3. Global site navigation

The language website has one primary entrance and a small, stable global nav:

1. **Documentation** — the complete learning and reference system.
2. **Playground** — edit and run Seseragi.
3. **Examples** — verified, task-oriented programs.
4. **Releases** — language and toolchain release notes.
5. **GitHub** — repository and issue tracker.
6. **Language switch** — English / 日本語.

There is no Community destination and no package-registry destination until a
real product exists for either one.

## 4. Documentation landing page

`/docs/` is a router, not a compressed language tutorial. It presents six
reader paths with short descriptions and recent-version context:

- Get Started
- Language Reference
- Standard Library
- Application Development
- Interop and Projects
- Tooling and Internals

The page also links to Playground, Examples, the normative Specification, and
the current release. It does not attempt to explain the language on the landing
page.

## 5. Documentation shell

Desktop uses three columns: section sidebar, article, and in-page table of
contents. Mobile turns the sidebar and table of contents into separate drawers.

The left sidebar has four levels:

1. documentation area;
2. collapsible subject group;
3. leaf article;
4. generated symbol children where applicable.

The active path stays expanded. Previous/next navigation follows the same order
as the sidebar. Search indexes leaf pages, headings, symbols, diagnostics, and
spec anchors; results identify whether a hit is Guide, Reference, API, or
Internals.

## 6. Get Started sidebar

```text
Get Started
├─ Overview
├─ Install the toolchain
├─ Hello, Seseragi
├─ Create a project
├─ Project layout
├─ Run a program
├─ Format and check code
├─ Test a project
├─ Build for production
├─ Create a Web application
└─ Where to go next
```

This is the only deliberately linear section. Each step is a separate page and
uses verified commands or source files.

## 7. Language Reference sidebar

### Language model

```text
Language Reference
├─ Language model
│  ├─ What Seseragi is
│  ├─ Design principles
│  ├─ Programs and entry points
│  └─ Features the language does not have
```

### Syntax and operators

```text
├─ Syntax and operators
│  ├─ Source text and comments
│  ├─ Literals
│  ├─ Characters, strings, and template escapes
│  ├─ Layout and blocks
│  ├─ Function application
│  ├─ Method calls
│  ├─ Pipelines and low-precedence application
│  ├─ Built-in operators
│  ├─ Custom operators and fixity
│  ├─ Reserved words
│  └─ Optional record field syntax
```

### Type system

```text
├─ Type system
│  ├─ Type-system properties
│  ├─ Built-in types
│  ├─ Type constructors
│  ├─ Type annotations and inference
│  ├─ Polymorphism
│  ├─ Nominal and structural types
│  ├─ Optional record fields
│  ├─ Closed structural records
│  ├─ Requirement merge
│  ├─ Function types and currying
│  ├─ Type identity and coercion
│  ├─ Recursive declarations
│  ├─ Kinds and type constructors
│  ├─ Type-parameter scope
│  ├─ Generic functions
│  ├─ Let-polymorphism and rank
│  ├─ Generic ADTs
│  ├─ Generic structs
│  ├─ Generic impls and methods
│  ├─ Generic aliases
│  ├─ Newtypes
│  ├─ Variance
│  └─ Type erasure and runtime representation
```

### Data, expressions, and patterns

```text
├─ Data, expressions, and patterns
│  ├─ Evaluation order
│  ├─ Let bindings and blocks
│  ├─ Binding rules
│  ├─ Irrefutable patterns
│  ├─ Conditional expressions
│  ├─ Algebraic data types
│  ├─ Structs
│  ├─ Newtypes
│  ├─ Records
│  ├─ Tuples, arrays, and lists
│  ├─ Ranges and comprehensions
│  ├─ Match and patterns
│  ├─ Lambdas
│  ├─ Impls and methods
│  └─ Struct and newtype operator overloads
```

### Traits and generic abstractions

```text
├─ Traits and generic abstractions
│  ├─ Why traits exist
│  ├─ Trait declarations
│  ├─ Instances
│  ├─ Constraints
│  ├─ Calling trait methods
│  ├─ Coherence and orphan rules
│  ├─ Standard operators and traits
│  ├─ Laws
│  ├─ Deriving
│  ├─ Methods versus trait methods
│  ├─ Do notation
│  ├─ Do desugaring
│  └─ Typing a do block
```

### Failure, Effect, and state

```text
├─ Failure, Effect, and state
│  ├─ Pure expressions
│  ├─ Maybe
│  ├─ Either
│  ├─ Effect<R, E, A>
│  ├─ Effect functions
│  │  ├─ Contract form
│  │  ├─ Compact inferred form
│  │  └─ Effectful for
│  ├─ Environment requirements
│  ├─ Typed error channels
│  ├─ Task
│  ├─ Sequential and parallel execution
│  ├─ Runtime execution boundaries
│  ├─ Defects
│  ├─ Cancellation
│  ├─ Resource scopes and finalizers
│  ├─ Scheduler fairness
│  ├─ Fiber supervision
│  ├─ Signal and MutableSignal
│  ├─ Derived signals
│  ├─ Signal read and update operators
│  ├─ Subscriptions and lifetime
│  └─ Exceptions and algebraic effects
```

### Modules

```text
└─ Modules
   ├─ Module units and identity
   ├─ Packages
   ├─ Top-level declarations
   ├─ Visibility
   ├─ Imports
   ├─ Module specifier resolution
   ├─ Re-exports
   ├─ Namespaces and name resolution
   ├─ Dependency graphs and cycles
   ├─ Initialization and evaluation
   └─ Entry points
```

`Maybe`, `Either`, `Effect`, `Task`, cancellation, resources, Fiber, and Signal
are separate leaf articles. The sidebar group merely keeps them discoverable.

## 8. TypeScript Interop sidebar

```text
Interop and Projects
├─ TypeScript interop
│  ├─ Boundary principles
│  ├─ Foreign modules
│  ├─ Pure foreign calls
│  ├─ Task foreign calls
│  ├─ Arity and currying
│  ├─ Boundary types
│  ├─ Primitive conversion
│  ├─ Collections and records
│  ├─ Optional, nullable, and rest parameters
│  ├─ Overloads
│  ├─ Classes, constructors, methods, and properties
│  ├─ Callbacks and lifetime
│  ├─ Calling Seseragi from TypeScript
│  ├─ ABI stability
│  ├─ Generic public ABI
│  └─ Source maps and cross-language stacks
```

## 9. `.d.ts` binding generation sidebar

```text
├─ Binding generation
│  ├─ What the converter generates
│  ├─ Inputs and symbol resolution
│  ├─ Generated outputs and reports
│  ├─ Primitive type conversion
│  ├─ Nullability and optional values
│  ├─ Collections, tuples, and objects
│  ├─ Functions
│  ├─ Overload selection
│  ├─ Generic declarations
│  ├─ Classes and enums
│  ├─ Unions and intersections
│  ├─ Unsupported TypeScript types
│  ├─ Discriminated-union opt-in
│  ├─ Updating declarations
│  ├─ Converter configuration
│  ├─ Callback lifetime configuration
│  ├─ Generated naming
│  └─ Declaration merging and namespace exports
```

## 10. Packages and projects sidebar

```text
└─ Packages and projects
   ├─ Manifest reference
   ├─ Package identity and versions
   ├─ Standard project layout
   ├─ Module paths
   ├─ Export maps
   ├─ Dependencies
   ├─ Package-local imports and tests
   ├─ Generated bindings
   ├─ Foreign host inputs
   ├─ Executable entry points
   ├─ Web documents and public assets
   ├─ Lockfiles
   ├─ Workspace discovery
   └─ Logical project inputs and loader adapters
```

## 11. Standard Library sidebar

The sidebar exposes every standard module. Each module landing page explains
semantics and cost; compiler-owned children document the exact public surface.

```text
Standard Library
├─ Overview
├─ Common API rules
├─ Prelude
├─ Data and validation
│  ├─ std/maybe
│  ├─ std/either
│  └─ std/validation
├─ Collections
│  ├─ std/collection
│  ├─ std/array
│  ├─ std/list
│  ├─ std/non-empty-list
│  ├─ std/map
│  └─ std/set
├─ Text
│  ├─ std/text
│  ├─ std/regex
│  ├─ std/text/grapheme
│  └─ std/text/unicode
├─ Numbers and bytes
│  ├─ std/number
│  ├─ std/int
│  ├─ std/float
│  ├─ std/math
│  ├─ std/big-int
│  ├─ std/decimal
│  ├─ std/bytes
│  ├─ std/bytes/hex
│  └─ std/bytes/base64
├─ Data formats
│  └─ std/json and decoders
├─ Time and randomness
│  ├─ std/time
│  ├─ std/clock
│  ├─ std/random
│  └─ std/entropy
├─ Effects and concurrency
│  ├─ std/effect
│  ├─ Temporal control
│  ├─ Resource scopes
│  ├─ Concurrency
│  ├─ std/ref
│  ├─ std/deferred
│  ├─ std/queue
│  └─ std/semaphore
├─ Streams and signals
│  ├─ std/stream
│  ├─ Demand, buffers, and overflow
│  ├─ Time operators
│  ├─ Terminal operations and resources
│  ├─ Stream-to-Signal conversion
│  └─ std/signal
├─ System capabilities
│  ├─ std/console
│  ├─ std/logger
│  ├─ std/stdin
│  ├─ std/filesystem
│  ├─ std/process
│  ├─ Termination and graceful shutdown
│  └─ Child processes
├─ Networking and browser
│  ├─ std/http
│  ├─ std/http/server
│  ├─ std/web/file
│  ├─ std/http/multipart
│  ├─ std/sse
│  ├─ std/websocket
│  ├─ std/websocket/server
│  ├─ std/web/navigation
│  └─ std/web/storage
├─ Testing and measurement
│  ├─ std/test
│  └─ std/benchmark
└─ Optional adapters
```

Generated children beneath a module use stable compiler identities, for
example `std/array` → types → traits → functions → instances. They are not
hand-copied signatures.

## 12. Application Development sidebar

### Web UI

```text
Application Development
├─ Web UI
│  ├─ Web module boundaries
│  ├─ Html values and children
│  ├─ Props records
│  ├─ Event actions
│  ├─ IME composition
│  ├─ Safe tags, attributes, styles, and URLs
│  ├─ Pure tree semantics
│  ├─ Stateful feature ownership
│  ├─ Server-side rendering
│  ├─ Dom services and targets
│  ├─ Event dispatch and resource lifetime
│  ├─ Signal-driven DOM binding
│  ├─ Large-scene application patterns
│  ├─ Hydration
│  ├─ Targets and interop
│  ├─ Document metadata
│  └─ SVG and browser interaction capabilities
```

### Data, servers, and storage

```text
├─ Data and protocols
│  ├─ JSON decoding
│  ├─ Bytes and encoding
│  ├─ HTTP clients
│  ├─ HTTP servers
│  ├─ Multipart and files
│  ├─ Server-sent events
│  └─ WebSockets
├─ Host applications
│  ├─ Console and structured logging
│  ├─ Filesystem
│  ├─ Processes and shutdown
│  ├─ Time, clocks, and randomness
│  ├─ Browser navigation
│  └─ Browser storage
└─ Databases
   ├─ PostgreSQL
   │  ├─ Identity and requirements
   │  ├─ Resources and values
   │  ├─ Query results and row decoding
   │  ├─ Transactions
   │  ├─ Cursors
   │  └─ Failures and Provider boundaries
   └─ SQLite
      ├─ Identity and requirements
      ├─ Databases and parameters
      ├─ Row decoding
      ├─ Transactions and resources
      └─ Failures, targets, and streams
```

## 13. Tooling sidebar

User-facing tool pages and implementation contracts are separated. The first
group explains how to use the tools; the second records exact shared behavior.

```text
Tooling
├─ CLI
│  ├─ run
│  ├─ build
│  ├─ test
│  ├─ format
│  ├─ lock
│  └─ target capabilities
├─ Editor support
│  ├─ VS Code extension
│  ├─ Language server
│  ├─ Diagnostics
│  ├─ Type inference explanations
│  ├─ Exhaustive-match fixes
│  ├─ Syntax highlighting
│  ├─ Document comments
│  └─ Deprecation metadata
├─ Playground
│  ├─ Virtual workspaces
│  ├─ Project compiler boundary
│  ├─ Spec fixtures and availability
│  ├─ Explorer
│  ├─ Editor tabs
│  ├─ Workspace execution
│  └─ Fullscreen HTML Preview
└─ Tool contracts
   ├─ Shared syntax pipeline
   ├─ Raw operator tokens
   ├─ Header scanning and module interfaces
   ├─ Flat operator chains and fixity resolution
   ├─ Incomplete source recovery
   ├─ Formatter contract
   ├─ Conformance cases
   ├─ Shared Analysis API
   ├─ Shared type documents
   ├─ Type display surface
   ├─ Diagnostic contract
   ├─ Test runner contract
   └─ Public-surface coverage
```

## 14. Internals sidebar

### Performance and production artifacts

```text
Internals
├─ Performance model
│  ├─ Performance guarantees
│  ├─ Observability and the as-if rule
│  ├─ Abstraction cost classes
│  ├─ Erased surface abstractions
│  ├─ Functions, currying, and traits
│  ├─ Data, collections, and fusion
│  ├─ Recursion and stack safety
│  ├─ Effect, Stream, and Signal costs
│  ├─ HTML and DOM costs
│  ├─ Build profiles
│  ├─ Verification and benchmarks
│  ├─ Production artifact manifests
│  ├─ Application reachability
│  ├─ Official runtime retention
│  ├─ Production bundle layout
│  ├─ Minification and source maps
│  └─ Artifact regression gates
```

### Runtime Provider Contract

The Provider section retains the specification's full leaf depth. It is split
into navigable groups, not collapsed into one provider article.

```text
└─ Runtime Provider Contract
   ├─ Model
   │  ├─ Purpose and source of truth
   │  ├─ Service identity and requirements
   │  ├─ Contract versions
   │  ├─ Logical types
   │  ├─ Operation identity and kinds
   │  ├─ Portable operations and target extensions
   │  └─ Capability validation examples
   ├─ Discovery and selection
   │  ├─ Provider manifests
   │  ├─ Requirement collection and visibility
   │  ├─ Compatibility filters
   │  ├─ Deterministic selection
   │  ├─ Preflight diagnostics
   │  └─ Lockfile and build metadata
   ├─ TypeScript runtime ABI
   │  ├─ Runtime ABI v1
   │  ├─ Logical-value projection
   │  ├─ Null, undefined, and missing
   │  ├─ Operation calls and result envelopes
   │  ├─ Opaque handles
   │  ├─ Conversion responsibility
   │  └─ Cross-capability projection
   ├─ Lifecycle
   │  ├─ Provider lifecycle contract
   │  ├─ Cold Effect and single start
   │  ├─ Terminal outcomes
   │  ├─ Cancellation races
   │  ├─ Resource acquire and handoff
   │  ├─ Scope cleanup and shutdown order
   │  └─ Causal metadata
   ├─ Callback and stream contract
   │  ├─ One-shot and multi-shot callbacks
   │  ├─ Registration and removal
   │  ├─ Demand and backpressure
   │  ├─ Overflow and protocol violations
   │  ├─ Producer and consumer cancellation
   │  ├─ Stream and Signal conversion
   │  └─ Capability boundaries
   ├─ Compatibility and conformance
   │  ├─ Portable surface and target extensions
   │  ├─ Independent version roles
   │  ├─ Additive and breaking changes
   │  ├─ Handshake diagnostics
   │  ├─ Provider conformance cases
   │  ├─ Backend replacement
   │  └─ Capability-level final validation
   └─ Application bridges
      ├─ Application capability baseline
      └─ HTTP server effectful-handler bridge
```

Every rejection-condition section in the Provider specification is a separate
leaf checklist beside the contract it validates. The Provider sidebar therefore
also includes Logical schema rejections, TypeScript ABI rejections, Lifecycle
schema rejections, Stream schema rejections, and Compatibility schema
rejections. Each page retains a direct normative anchor and links every rejected
shape back to the page that explains the corresponding valid contract.

## 15. Examples and Cookbook

Examples are task-oriented and do not replace reference pages.

```text
Examples
├─ Language examples
├─ Effect and concurrency
├─ Web interfaces
├─ HTTP and data
├─ Resources and cancellation
├─ TypeScript interop
├─ PostgreSQL
├─ SQLite
└─ Testing

Cookbook
├─ Decode and validate external data
├─ Model domain failures
├─ Compose environment services
├─ Run work in parallel
├─ Acquire and release resources
├─ Build a form
├─ Bind Signal state to DOM regions
├─ Stream server events
├─ Handle graceful shutdown
└─ Test with deterministic services
```

## 16. Leaf article contract

Every reference article follows the same content contract. Empty headings are
not emitted when a subject does not have that dimension.

1. concise definition;
2. syntax or public signature;
3. typing and inference rules;
4. evaluation order and runtime semantics;
5. failure, cancellation, and resource behavior where relevant;
6. visibility, package, interop, or Provider boundary where relevant;
7. performance or complexity guarantees where specified;
8. verified examples and counterexamples;
9. diagnostics and invalid forms;
10. related pages;
11. exact links to normative specification sections.

This contract prevents a page from becoming a decorative summary. A reader can
use a leaf article to answer what is valid, what type it has, when it executes,
how it fails, and which boundary owns the behavior.

## 17. Visual-design consequences

- Section landing pages use generous whitespace and a short route list; they do
  not preview every concept in marketing cards.
- Reference pages prioritize the sidebar, article typography, code, tables, and
  direct links to the specification.
- Code is rendered as real highlighted text with copy and Playground actions,
  never as an editor screenshot.
- The desktop sidebar must demonstrate at least three expanded levels in the
  next mockup, because depth is part of the product design.
- The next mockup set must include `/docs/`, one deep language reference page,
  one generated Standard Library symbol page, one Web guide, and one Tooling or
  Provider contract page. A homepage alone cannot validate this architecture.
