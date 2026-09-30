# Seseragi reference site architecture

Status: design source for the public Seseragi reference site. This document
defines how readers navigate the language. `docs/spec/` remains normative.

## 1. Product boundaries

Seseragi has three different reading experiences. They must not be collapsed
into one giant documentation tree.

| Surface | Reader question | Organization |
| --- | --- | --- |
| Tour | How do I learn Seseragi in order? | One interactive sequence |
| Language Reference | What does this language concept mean and what are its exact rules? | Concepts and semantic relationships |
| Standard Library | Which module or symbol provides this operation? | Modules and compiler-owned symbols |

The Tour at `https://seseragi.vercel.app/tour/` is the only ordered learning
course. The reference site links to it but does not recreate a Get Started or
Learn track.

Examples, releases, and the Playground are global destinations. Tooling,
package, interop, Web, and runtime-provider material may gain their own
reference surfaces when the content exists. They must not be inserted into the
Language Reference or Standard Library sidebars merely to make a complete
looking site.

There is no Community or package-registry destination until a real product
exists for it.

## 2. Content ownership

- `docs/spec/` defines normative behavior.
- The public reference explains that behavior in reader-facing language. A
  reader should not need to decode the specification to understand a feature.
- `examples/spec/` and site-owned verification sources provide executable
  examples. Prose and code panels do not copy Tour lessons.
- Compiler metadata owns Standard Library module, symbol, signature, instance,
  and availability data.
- English is canonical. Japanese has the same page identity and hierarchy under
  `/ja/`; incomplete translations are identified explicitly.

## 3. Human-centred page boundaries

A specification section is evidence, not automatically a page.

Create a leaf page when a reader can reasonably arrive with one distinct
question, the answer has a stable name, and the page can explain a coherent
contract. A page may cover several normative subsections. One normative
subsection may also support several pages when readers encounter it through
different concepts.

Do not combine independent concepts merely because they share an implementation
or a specification chapter. In particular, failure, Effect, Task,
cancellation, resources, Signal, modules, and interop remain separately
addressable topics.

Every article follows `reader-contract.md` and answers, in the order useful to a reader:

1. what the concept is and when it matters;
2. its accepted form, with a small verified example;
3. how to read the example and why its output or inferred type follows;
4. its runtime or evaluation behavior;
5. invalid forms and the diagnostics they produce;
6. interactions, limits, costs, and related concepts;
7. necessary type rules and purpose-labelled optional links for deeper questions.

The normative specification sections are internal provenance, not a public
filler section. Unexplained vocabulary and prerequisite links do not satisfy
these questions. Introduce the necessary ideas locally before advanced rules.

Not every heading is required on a short page, but none of the applicable
questions may be silently omitted.

## 4. Coverage without specification-shaped navigation

Completeness is tracked in an internal coverage map rather than exposed as the
site hierarchy. Each normative rule maps to one or more public pages and to a
visible article location. Coverage records include:

- syntax and rejected syntax;
- type inference, constraints, identity, and coercion;
- evaluation order and desugaring;
- success, typed failure, defects, and cancellation;
- resources, ownership, subscription, and lifetime;
- visibility, module identity, packages, and interop boundaries;
- performance, allocation, stack safety, and ordering guarantees;
- diagnostics and invalid configurations;
- verified examples and counterexamples.

A route list is not proof of coverage. A specification link is provenance, not
a substitute for an explanation.

## 5. Global navigation

The public site uses a small stable header:

1. Documentation — opens the reference index;
2. Playground — edits and runs Seseragi;
3. Tour — the ordered learning path;
4. Examples — verified complete programs;
5. Releases — language and toolchain history;
6. locale switch — English / 日本語.

## 6. Reference index

`/docs/` is the entrance to reference material, not a compressed tutorial. It
contains three calm routes:

- Language Reference;
- Standard Library;
- the external Tour link for readers who want a sequence.

It does not render the article sidebar. It does not explain Effect, types, Web,
or tooling on the landing page.

## 7. Navigation shell

Language and library articles use separate navigation trees. An article sidebar
renders only the tree containing the current page.

Desktop uses:

1. the current reference tree;
2. the article;
3. the current article's table of contents.

Mobile opens the current reference tree as a left off-canvas sidebar above the
article, not an accordion that pushes the article down. A compact sticky menu
trigger remains available while reading. The drawer scrolls independently,
locks background scrolling, and closes with its close button, backdrop tap,
Escape, or page selection. Native dialog modality isolates keyboard focus;
closing restores the reading position. The article table of contents remains a
separate compact disclosure. The same off-canvas tree is usable without
JavaScript. Collapsed branches do not render thousands of symbol links. The
active group and section remain expanded.

Previous/next links stay within the current concept cluster or library module.
They must not flatten the full sidebar into a supposedly ordered reading course.
Show the current cluster and an overview link; related destinations explain what
the reader can find there.

The navigation levels are:

1. reference surface;
2. reader-oriented concept group or API category;
3. concept cluster or module;
4. leaf concept or generated symbol.

## 8. Language Reference sitemap

The sitemap below is the target information architecture. Routes may be filled
incrementally, but completed pages must be placed in this reader model rather
than in a temporary Learn or miscellaneous group.

```text
Language Reference
├─ Understand Seseragi
│  ├─ What Seseragi is
│  ├─ Programs and entry points
│  ├─ Design principles
│  └─ Features Seseragi deliberately does not have
├─ Source and syntax
│  ├─ Source text and comments
│  ├─ Names and reserved words
│  ├─ Literals
│  ├─ Characters, strings, and template escapes
│  ├─ Layout, blocks, and line continuation
│  └─ Optional record field syntax
├─ Functions and operators
│  ├─ Function declarations
│  ├─ Parameters and currying
│  ├─ Function application
│  ├─ Method calls
│  ├─ Pipelines and low-precedence application
│  ├─ Built-in operator precedence
│  └─ Custom operators and fixity
├─ Values and data modelling
│  ├─ Bindings and immutability
│  ├─ Algebraic data types
│  ├─ Structs
│  ├─ Newtypes
│  ├─ Records
│  ├─ Tuples, arrays, and lists
│  └─ Ranges and comprehensions
├─ Expressions and control flow
│  ├─ Evaluation order
│  ├─ Blocks
│  ├─ Conditional expressions
│  ├─ Match expressions
│  ├─ Lambdas
│  └─ Impls and methods
├─ Patterns
│  ├─ Binding and irrefutable patterns
│  ├─ Constructor patterns
│  ├─ Record, tuple, array, and list patterns
│  ├─ Guards and exhaustiveness
│  └─ Diagnostics for unreachable or incomplete matches
├─ Types
│  ├─ Type-system model
│  ├─ Built-in types
│  ├─ Type constructors and kinds
│  ├─ Annotations and inference
│  ├─ Polymorphism and rank
│  ├─ Nominal and structural types
│  ├─ Records and requirement merge
│  ├─ Function types
│  ├─ Identity and coercion
│  ├─ Recursive declarations
│  ├─ Generics and type-parameter scope
│  ├─ Variance
│  └─ Runtime representation
├─ Traits and implementations
│  ├─ Trait declarations and constraints
│  ├─ Instances and method resolution
│  ├─ Coherence and orphan rules
│  ├─ Laws and deriving
│  ├─ Operator traits
│  └─ Do notation and desugaring
├─ Failure and Effect
│  ├─ Pure expressions
│  ├─ Maybe and Either
│  ├─ Effect<R, E, A>
│  ├─ Requirements and typed error channels
│  ├─ Effect function forms
│  ├─ Execution boundaries
│  └─ Defects
├─ Concurrency and resources
│  ├─ Task
│  ├─ Sequential and parallel execution
│  ├─ Cancellation
│  ├─ Resource scopes and finalizers
│  ├─ Fibers and supervision
│  └─ Scheduler guarantees
├─ Signals
│  ├─ Signal and MutableSignal
│  ├─ Derived signals
│  ├─ Reading and updating
│  └─ Subscriptions and lifetime
├─ Modules and packages
│  ├─ Module identity
│  ├─ Top-level declarations and visibility
│  ├─ Imports, exports, and resolution
│  ├─ Dependency graphs and initialization
│  ├─ Packages, manifests, and lockfiles
│  └─ Entry points and targets
├─ TypeScript interop
│  ├─ Boundary model and foreign modules
│  ├─ Calls, currying, and callbacks
│  ├─ Type conversion and overloads
│  ├─ Classes and object boundaries
│  ├─ Public ABI and source maps
│  └─ Binding generation
└─ Grammar and diagnostics
   ├─ Grammar reference
   ├─ Diagnostic catalogue
   └─ Source ranges and recovery
```

## 9. Standard Library sitemap

The library tree is generated from compiler metadata. Categories help a reader
find a module; module pages explain semantics, availability, target support,
and cost; generated children document exact public types, traits, constructors,
functions, operators, and instances.

```text
Standard Library
├─ Prelude
├─ Data and validation
├─ Collections
├─ Text and Unicode
├─ Numbers and bytes
├─ Data formats
├─ Time and randomness
├─ Effects and concurrency
├─ Streams and signals
├─ System capabilities
├─ Networking and browser
└─ Testing and measurement
```

Generated symbol routes use stable compiler identities. Signatures are never
hand-copied into a second source of truth.

## 10. Cross-reference rules

- A Language Reference page may link to relevant library modules and symbols,
  but does not embed the library tree.
- A library symbol may link back to the language concepts needed to understand
  its signature or behavior.
- A reference article may link to a Tour step as an optional exercise. It does
  not depend on the Tour for its explanation.
- Examples may show a complete application, while reference pages keep examples
  minimal and specific to the rule being explained.
- Tooling and provider implementation details are linked only where they change
  user-visible language or runtime behavior.

## 11. Review gates

A documentation change is complete only after these reviews:

1. **Information architecture** — the page answers a stable reader question and
   appears in the correct reference tree.
2. **Specification coverage** — every claimed rule has normative evidence and
   no applicable rule was hidden by an overview.
3. **Executable truth** — valid and invalid examples are checked at their owning
   compiler, runtime, or tool boundary.
4. **Navigation** — the current branch is visible without exposing unrelated
   reference trees or every generated symbol.
5. **Visual QA** — desktop and mobile are inspected for type scale, line length,
   whitespace, code readability, overflow, and locale behavior.
6. **Dead-content audit** — replaced pages, routes, samples, styles, and tests
   are removed instead of left as a second architecture.
