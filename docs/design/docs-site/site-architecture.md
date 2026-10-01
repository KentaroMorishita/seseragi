# Seseragi reference site architecture

Status: design source for the public Seseragi reference site. This document
defines how readers navigate the language. `docs/spec/` remains normative.
The #697 revision makes the everyday TypeScript reader in
[reader-contract.md](reader-contract.md) explicit. It is a design input for
#698–#702; the page changes below are not yet implemented or reader-accepted.

## 1. Product boundaries

The Docs themselves must let a practical TypeScript developer understand what
Seseragi is for, see a fair same-task comparison, try a small program, and find
both approachable explanations and exact rules. The Tour is optional.

| Surface | Reader question | Responsibility and next destinations |
| --- | --- | --- |
| Home | What is this language, and why might I try it? | A concrete use, a verified small TS/Seseragi comparison and current limits; link to the introduction, first run and Docs |
| Docs index | Where do I start, or find an answer? | Brief purpose/benefit context, an obvious practical starting path, and the existing Language/Library navigation hub |
| Introduction | How would this change familiar code? | Explain the comparison and new notation, uses and limitations; link to first run, basic concepts and deeper rules |
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
with the responsibilities above. The #697 pilot needs one first-run task page,
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
language is for, a concrete benefit backed by the selected comparison, and
purpose-labelled routes to the introduction and first run. A reader arriving
here must not have to choose between a wall of unfamiliar concepts and Tour.

Keep this a navigable entrance rather than a compressed course: the introduction
owns the comparison walkthrough, the first-run page owns commands, and concept
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
7. **Reader understanding** — apply the TS-reader contract and record remaining
   questions separately from technical checks; self-review does not establish
   actual first-time-reader acceptance.

## 12. Entrance and pilot page map

Inspected against `ab42da841d76d322584fac096c249d586ae539ec`. Existing source
paths below are relative to `apps/site/src/pages/`; each owns `page.ssrg`,
`ja.ssrg` and `en.ssrg` (and `guide.ssrg` where present). Preserve its `id` and
route. These are implementation assignments, not additional accepted pages.

### Entrance and practical setup

| Route / page source | Work owner and page job | Useful onward links |
| --- | --- | --- |
| `/` — `home/` | #700: purpose, a small verified #698 comparison, current limitations | Introduction; first run; Docs |
| `/docs/` — `docs/overview/` | #700: short purpose/benefit context and start choices before the retained #675 reference groups | Introduction; first run; Language; Library; optional Tour |
| `/docs/language/model/what-is-seseragi/` — `language/model/what-is-seseragi/` | #700: explain the same-task comparison without assuming ADT/match/Effect; describe supported uses and limits | First run; values; function calls; relevant detailed rules |
| `/docs/first-run/` — proposed `docs/first-run/`, `id: docs.first-run` | #699: new bounded task page; install/verify, create a file, check, run, inspect output, fix representative failures | Values and names; declaring/calling a function; existing release information |
| `/docs/language/` — `language/overview/` | #700: keep the concept index; make the basic value/function destinations recognizable, without a second lesson sequence | The existing concept groups and selected pilot pages |

`/docs/first-run/` and its `/ja/` mirror are planned, not live. #699 adds them to
the typed catalog outside the two reference trees, using the existing standalone
landing/block components, and updates the build/navigation checks. In particular,
revise the current fixed output count and Tour-only entrance assertions, and add
first-run coverage in both locales at desktop/mobile sizes and without
JavaScript. Do not add fake specification coverage to count a practical procedure
among the 351 mapped reference leaves. The page does not require readers to study modules, package
graphs or Effect before running the program. Explain each wrapper line and the
chosen execution target in place. Existing function/block samples are currently
verified with an injected entry harness; they are not yet standalone programs
with visible output. #699 must verify the exact complete program the reader
copies, not count that existing harness as a clean first run.

#699 verifies the currently published installation route, supported OS/CPU and
version, actual prerequisites, PATH, working directory, filenames and output in
a clean user environment. Do not prescribe an unverified installer here.
`README.md` remains the repository's short orientation and quick links;
`docs/GETTING_STARTED.md` currently owns the **local Web app** walkthrough, not
the minimal first process run. Keep that distinct task and link it after first
success when useful. Release documentation owns version-specific distribution
facts. The first-run page owns its complete minimal procedure, reusing verified
commands/sources rather than maintaining competing quick starts. #699 must
reconcile overlapping README steps with this ownership when implementing it.
Existing Projects/Tooling coverage in #629 remains open; do not implement or
claim acceptance of those full surfaces to deliver one first run.

### Values and functions pilot (#701)

Six existing articles form the bounded pilot. Their opening examples use only
the local prerequisites in the reader contract; advanced sections stay available
on the same pages. No new “values course” or function-declaration leaf is needed
for this pilot.

| Route (under `/docs/language/`) / matching `language/` source | Opening question and local explanation | Details to retain after the basic case |
| --- | --- | --- |
| `model/immutable-by-default/` | How do I name a value and make an updated value? Start with plain `let` and familiar TS values; distinguish binding from object immutability | Collection behavior and explicit state APIs |
| `types/built-in-types/` | Which type describes this value? Introduce the numbers, strings and booleans the examples actually use | Char, numeric bounds/conversion, Unit/Never and runtime restrictions |
| `types/annotations-and-inference/` | Where do I write a type? Own the first ordinary `fn` declaration: name, parameters, each arrow, return type and expression body, plus a simple annotated `let` | Public declarations, inference limits and the separately explained `effect fn` exception |
| `syntax/function-application/` | How do I call that function? Restate a fully defined small declaration, supply all arguments, read the result and fix a TS-style call | Partial application, Unit calls, explicit type arguments, precedence and special forms |
| `expressions/blocks-and-local-declarations/` | How do several lines produce a result? Explain local names, scope and the final expression | Recursive/local declarations, constraints and distinction from `do` |
| `types/function-types-and-currying/` | What happens when I supply only some arguments? Start after an ordinary two-argument call, define the remaining function and then name currying | Association, type rules and anonymous Unit-parameter behavior |

Keep the existing `language.types.function-types` page ID for the
`types/function-types-and-currying/` route; do not derive a replacement ID from
the URL. The other pilot IDs likewise remain those in their page modules.

The introduction and first run can point directly to the value and function
articles. Between clusters, labels should answer a purpose such as “Write a
function and its types” or “Call a function”; include the destination's localized
page title rather than inventing a second title. Within a page, headings should
separate the task/example/result from exact rules and uncommon cases. A table of
contents lets experienced readers jump to those rules. No fixed heading count
or forced template is required.

Supporting pages such as `syntax/literals/`, `patterns/binding-rules/`,
`data/records/` and `expressions/conditionals/` retain their identities and are
linked for their specific questions; they are not silently added to the pilot.
Summarize any piece needed by a pilot example locally. Prefer a simpler example
if it would otherwise require rewriting those topics first. In particular,
plain `let` does not require the nested-pattern lesson currently on binding rules.

### After the pilot

#702 first checks the entrance, first run and these six articles against the
reader questions. Only then choose the next implementation scope from existing
`data/algebraic-data-types/`, `patterns/match/`, `effects/maybe/`,
`effects/either/` and, when needed, the separate Effect/failure articles. Introduce
data alternatives before matching and begin failure explanations from the
reader's `try` / `catch` experience. These are dependency candidates, not approval
to rewrite them or a claim that the remainder of #628/#630/#629 is accepted.

#698 can now choose and verify everyday comparison candidates; #699 can verify
the minimal installation/run path in parallel. Their evidence feeds #700's
entrance, then #701's pilot and #702's reading review. No comparison, OS support
or setup success is claimed by this design-only change.
