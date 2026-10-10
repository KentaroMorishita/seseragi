# Seseragi reference site architecture

Status: information-architecture input for Docs Reboot #601. The sole editorial
policy is [editorial-contract.md](editorial-contract.md); this file owns site
surfaces and navigation. The existing hierarchy is migration material, not an
approved new content plan. `docs/spec/` remains normative. Retirement decisions
and implementation dependencies live in [migration-ledger.md](migration-ledger.md).

## 1. Product boundaries

The Docs themselves must let programmers, regardless of their previous language,
understand what Seseragi is for, read a small meaningful example, try a program
and find both approachable explanations and exact rules. The Tour is optional.

| Surface | Reader question | Responsibility and next destinations |
| --- | --- | --- |
| Home | What is this language, and why might I try it? | Hero, one verified fn/main example or demo, a short introduction and clear links to first run and Docs |
| Docs index | Where do I start, or find an answer? | Brief purpose/benefit context, an obvious practical starting path, and the existing Language/Library navigation hub |
| Introduction | How would this change familiar code? | Explain useful code and new notation, uses and limitations; use other-language comparisons only where helpful; link to first run, basic concepts and deeper rules |
| First run | How do I run my own first program? | A bounded install/check/run task with files, commands, result and recovery; finish at values/functions |
| Concept article | What does this mean and how do I use it? | Explain the simple case locally before detailed rules, even when reached directly |
| Detailed Language Reference / Standard Library | What are the exact rules, limits or APIs? | Preserve addressable concepts and compiler-owned symbols, with prerequisites summarized where needed |
| Tour | Can I learn by working through exercises in order? | The interactive course, linked as an optional practice path |

Concept explanations and detailed rules normally share the existing article
identity. “Approachable” and “detailed” are reading needs, not two duplicated
language trees. Headings and ordering may improve without changing route IDs.

#668 correctly removed a duplicated linear course and separated the reference
trees; #675 made `/docs/` a useful navigation hub. Preserve those results, but
replace the earlier prohibition on any Docs introduction or practical setup
with the responsibilities above. The first-run surface owns a bounded practical task,
not the old `/docs/get-started/` sequence. This does not ban other independently
useful task guides when their scope is justified.

Examples, releases, and the Playground remain global destinations. Tooling,
package, interop, Web, and runtime-provider references retain their separately
mapped responsibilities; do not fill the Language or Library trees with them
merely to make a complete-looking site. Root-domain cutover (#631) and Tour
redesign are outside this change.

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
- English keeps unprefixed canonical URLs. Japanese has the same page identity
  and hierarchy under `/ja/`. Public copy is authored Japanese-first, then
  English with the same examples, conditions, limits and results; URL ownership
  does not set the drafting language.

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

Article composition follows [the editorial contract](editorial-contract.md).
A short example can grow into a discovery before exact rules; applicable
semantics stay traceable without mandatory purpose/type/evaluation/diagnostic
headings. The specification map owns provenance, not public filler sections.
Summarize context needed by a deep-link reader locally. Chapter order and
purpose-labelled exits can evolve while exact-rule anchors remain addressable.

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

1. Documentation — opens the Docs index and its start/reference choices;
2. Playground — edits and runs Seseragi;
3. Tour — the ordered learning path;
4. Examples — verified complete programs;
5. Releases — language and toolchain history;
6. locale switch — English / 日本語.

## 6. Reference index

`/docs/` serves both a first-time visitor and someone looking up a rule. Retain
#675's visible Language Reference and Standard Library groups and its absence
of an article sidebar. Precede those groups with concise context: what the
language is for, a concrete benefit backed by verified code, and
purpose-labelled routes to the introduction and first run. A reader arriving
here must not have to choose between a wall of unfamiliar concepts and Tour.

Keep this a navigable entrance rather than a compressed course: the introduction
owns the language introduction, the first-run page owns commands, and concept
articles own explanations. Offer the Tour as optional exercises. Do not replace
the current useful groups with a long tutorial or hide reference lookup behind
onboarding. Only link to a new destination once its page exists in both locales.

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
Show the current cluster and an overview link. A prerequisite relationship across
clusters is a purpose-labelled related link, never a misleading Next button.
Section ordering and labels may change to put the simple case first; keep page
identities and verify breadcrumb, localized title and link text together.

The navigation levels are:

1. reference surface;
2. reader-oriented concept group or API category;
3. concept cluster or module;
4. leaf concept or generated symbol.

## 8. Language Reference sitemap

The sitemap below is the target information architecture. Routes may be filled
incrementally, but completed pages must be placed in this reader model rather
than in a temporary Learn or miscellaneous group. This sitemap describes
reference responsibilities, not a required order of study. The first-run task
sits outside the language/library trees; the concrete pilot map in section 12
uses implemented identities rather than creating every illustrative label below
as a new page.

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
7. **Reader understanding** — apply the editorial contract and reader-review protocol and record remaining
   questions separately from technical checks; self-review does not establish
   actual first-time-reader acceptance.

## 12. Entrance and pilot page map

This section replaces the old #698–#701 dispatch order. Those issues and their
verification files retain implementation history only. #601 owns dependencies:
#763 contract → #764 typed article composition → #765 Functions/Notation pilot;
#766 uses that chapter as its design reference. #767 bulk migration follows the
new composition and accepted chapter. #768 is optional after #764. Independent
#702/#706/#740/#631 gates remain separate.

| Surface / existing source | New owner and responsibility |
| --- | --- |
| `/` — `apps/site/src/pages/home/` | #766: Hero, one code/demo, short introduction, clear destinations |
| `/docs/` — `apps/site/src/pages/docs/overview/` | #766: entry by purpose and by technical name, same destination identity |
| `/docs/language/` — `apps/site/src/pages/language/overview/` | #766: chapter and feature entry points without duplicating Tour |
| `/docs/language/model/what-is-seseragi/` | #766: small useful code, local notation, current capabilities and limits |
| `/docs/first-run/` — `apps/site/src/pages/docs/first-run/` | #766: preserve the implemented installation/source/check/run path and reverify changed commands |
| `/examples/`, `/releases/` | #766: verified source and real release destinations; retain useful existing results |

These surfaces are implemented in the current catalog, with Japanese mirrors.
Their presence and old execution/browser evidence do not establish Reboot prose
acceptance. `first-run-verification.md`, `typescript-comparison.md` and
`examples-releases-verification.md` retain useful prior source and execution
history, rather than instructions to restart their old issues.

### Functions and notation pilot (#764 / #765)

Choose article count and chapter headings after reading the editorial contract.
Keep existing identities and detailed URLs where practical; do not create an
unnecessary duplicate chapter tree. The first #764 route is function application.

| Existing route below `/docs/language/` | Role in the pilot |
| --- | --- |
| `syntax/function-application/` | Write a small fn; ordinary application; first freely composed bilingual page in #764 |
| `types/function-types-and-currying/` | Functions as values, partial application, exact function-type rules |
| `expressions/lambdas/` | Small anonymous functions and higher-order use |
| `syntax/pipelines-and-low-precedence-application/` | Combine functions; choose ordinary application, $ or pipeline for the actual task |
| `syntax/operator-precedence/` | Addressable grouping and operator details; link from the chapter |
| `traits/do-notation/`, `effects/maybe/`, `effects/either/` | Optional exits toward mapped/applied/dependent composition; do not teach all theory inside the pilot |

Preserve `language.types.function-types` for the currying page and
`language.syntax.pipelines` for the pipeline page; identity is not derived from
URL spelling. The route inventory and template-removal prerequisites are in the
migration ledger. Existing values/types/blocks pages remain linked background,
not an expansion of the old six-page pilot or a new work queue.

The chapter must pass execution, bilingual meaning, author review, actual reader
review and browser/navigation checks separately. Never infer these results from
historical checked boxes. Each replaced page keeps its source/spec obligations,
anchors and locale identity until a validated replacement or redirect exists.
