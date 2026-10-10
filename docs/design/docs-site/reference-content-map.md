# Seseragi Reference Content Map

This file maps semantic obligations to existing or planned routes. It is a
provenance ledger, not an editorial policy or a published-route inventory.
[editorial-contract.md](editorial-contract.md) owns prose and composition;
[migration-ledger.md](migration-ledger.md) distinguishes live implementation
from planned-only rows. The specification remains the semantic review source.
Mapped/Outlined/Verified below are historical coverage states, not Docs Reboot
reader or author acceptance. Retain rules when moving or merging their owner.

## Coverage states

- **Mapped**: every rule in the source section has an article location.
- **Outlined**: the article has syntax, typing, semantics, invalid cases,
  examples, diagnostics, and cross-links where applicable.
- **Verified**: examples and signatures are sourced from canonical executable
  files or compiler metadata, and the article has been checked against the
  current specification.

No visual mockup represents a finished section until its leaves are at least
Outlined.

## Scope boundary

This ledger covers the Language Reference and its related reference surfaces.
The Tour owns the interactive course; the Docs also explain purpose, appeal,
basic concepts and a practical first run without requiring that course. See
[site architecture](site-architecture.md#12-entrance-and-pilot-page-map) for the
current Reboot entrance/pilot assignments. Introductory and first-run task pages do not
create additional normative coverage rows here. CLI, project, Web, and tooling
topics enter this map as independently useful reference contracts, not steps in
a duplicated course. Keep each existing route and its full semantic obligations
when moving its simpler explanation ahead of advanced rules.

## Language Reference / Language model

| Route | Source | Required semantic scope |
| --- | --- | --- |
| `/docs/language/model/what-is-seseragi/` | 0.1 | Expression-oriented static language; immutability; explicit absence/failure/async/effects; ADTs, exhaustive matching, typeclasses, modules, and explicit foreign boundaries. |
| `/docs/language/model/design-principles/` | 0.2 | Index and dependency map for the seven principles; no principle is explained only on this landing page. |
| `/docs/language/model/expression-oriented/` | 0.2 | Declarations and control forms producing values; Unit/Never behavior; links to block, conditional, match, and do typing. |
| `/docs/language/model/immutable-by-default/` | 0.2 | Immutable ordinary values, persistent data, Effect-managed external state, Signal/Ref capability boundaries, and the absence of mutable variables/fields. |
| `/docs/language/model/no-hidden-danger/` | 0.2 | Explicit absence, failure, Effect, nullish/foreign boundaries, numeric conversion, and rejected truthiness/implicit exceptions. |
| `/docs/language/model/backend-independent-semantics/` | 0.2 | Which semantics all backends preserve and which representations/optimizations are free; links to Provider and performance contracts. |
| `/docs/language/model/diagnosable-behavior/` | 0.2 | Stable rules, explicit ambiguity, no value-directed magic, shared tooling artifacts, and diagnostic ownership. |
| `/docs/language/model/visible-costs/` | 0.2 | Collection/effect/interop costs that must remain inferable from APIs and documentation; links to the performance model. |
| `/docs/language/model/readable-density/` | 0.2 | Currying/pipelines/type inference balanced with explicit boundaries; surfaces intentionally not added; formatter relationship. |
| `/docs/language/model/programs/` | 0.3 | Program composition from packages/modules/declarations/expressions; compilation and host execution boundary; main and library roles; current implementation status kept separate from language meaning. |
| `/docs/language/model/non-features/` | 0.4 | Every explicitly absent feature, the rule used instead, and links to the replacement model; absence must not be presented as an unimplemented placeholder. |

## Language Reference / Syntax and operators

### `/docs/language/syntax/source-text/` — Source text

Source: `docs/spec/01-syntax.md` 1.1.

The article must explain:

- UTF-8 source encoding;
- LF and CRLF equivalence;
- Unicode `XID_Start` and `XID_Continue` identifier rules;
- the permitted trailing apostrophe in identifiers;
- ASCII, case-sensitive keywords;
- Unicode identifier examples;
- `//` line comments and the absence of block comments;
- how apostrophes are distinguished from the start of a Char literal;
- invalid identifier examples and the lexer boundary involved.

Required cross-links: Reserved words; Character and string escapes; Tooling /
Shared syntax pipeline.

### `/docs/language/syntax/literals/` — Literals

Source: `docs/spec/01-syntax.md` 1.2, excluding the dedicated escape leaf.

The article must explain:

- the literal forms and resulting types for Int, Float, Bool, Char, String,
  template String, List, and Unit;
- why unary minus is not part of a numeric literal;
- why numeric literals do not use typeclass overloading;
- decimal, binary, octal, and hexadecimal Int spelling;
- digit-separator placement and rejected spellings;
- decimal-only Float spelling, exponent syntax, and rejected forms;
- the exact Int range and Float rounding/overflow/underflow behavior;
- `-0.0` versus the absence of Int negative zero;
- longest-match tokenization and `SES-P0203`;
- runtime defect behavior for overflowing Int arithmetic;
- the lexical distinction between a List literal and a template String;
- template interpolation typing through `Show<A>`;
- left-to-right, exactly-once interpolation evaluation;
- the fact that templates are pure and perform no implicit I/O;
- String as Unicode scalar values and why indexing is not built in.

The article needs valid and invalid examples for every numeric family, plus a
template example whose evaluation order is observable without introducing an
Effect.

Required cross-links: Character and string escapes; Built-in types; Show;
Numeric modules; Defects.

### `/docs/language/syntax/character-string-escapes/` — Character and string escapes

Source: `docs/spec/01-syntax.md` 1.2.1.

The article must explain:

- the exactly-one-Unicode-scalar rule for Char;
- combining scalars and why they do not form one Char;
- rejection of surrogate code points;
- identifier apostrophes versus Char delimiters;
- every supported common escape and delimiter-specific escape;
- unsupported hex, octal, named Unicode, and cross-delimiter escapes;
- the one-to-six digit `\u{...}` range and invalid values;
- raw-control-character restrictions in Char and String;
- raw newlines in templates and CRLF normalization;
- interpolation start, escaped `${`, and an unescaped standalone `$`;
- `SES-P0201`, `SES-P0001`, and `SES-P0202` ranges;
- the formatter guarantee that valid spelling and escapes preserve meaning.

Required examples include a scalar Char, a combining-sequence rejection,
delimiter escapes, literal `${`, and an unterminated-literal diagnostic.

### `/docs/language/syntax/layout-and-line-continuation/` — Layout and line continuation

Source: `docs/spec/01-syntax.md` 1.3.

The article must explain:

- why indentation never creates a block;
- braces as the block boundary;
- newline and semicolon as declaration/expression separators;
- free newlines inside parentheses;
- the exact two continuation conditions outside parentheses;
- operator-first multiline style for bind, pure let, and expressions;
- semicolon as an unconditional separator;
- why indentation width does not affect parsing;
- the formatter's two-space canonical representation;
- accepted and rejected multiline examples, including cases requiring
  parentheses.

Required cross-links: Blocks; Pipelines and low-precedence application;
Formatter contract; Incomplete-source recovery.

### `/docs/language/syntax/function-application/` — Function application

Source: `docs/spec/01-syntax.md` 1.4, `docs/spec/02-types.md` 2.7, and
`docs/spec/03-data-and-expressions.md` 3.1.

The article must explain:

- whitespace application and left associativity;
- the parse of `add 1 2` as `(add 1) 2`;
- the absence of `f(x, y)` multi-argument call syntax;
- the distinct roles of grouping, tuple, and Unit parentheses;
- explicit type arguments attached without whitespace;
- the parse difference between `identity<String> value` and
  `identity < value`;
- whitespace requirements for custom infix operators;
- operator values such as `(<+>)`;
- declarations with an implicit anonymous Unit parameter;
- the resulting `Unit -> A` type and the required `()` call;
- curried function types and partial application;
- left-to-right parameter binding;
- strict evaluation of arguments, left to right, exactly once;
- the fact that an Effect value is constructed but its body is not run by
  ordinary application;
- type errors for applying non-functions and for incomplete constraints;
- when grouping is required around an argument expression.

This is a full semantic article, not a two-line syntax sample. It needs parsing
diagrams for associativity, type progressions for partial application, and
evaluation-order examples.

Required cross-links: Function types and currying; Generic functions; Method
calls; Pipelines; Effect runtime boundaries.

### `/docs/language/syntax/method-calls/` — Method calls

Source: `docs/spec/01-syntax.md` 1.5, with resolution rules from the impl and
name-resolution sections.

The article must explain:

- `value.method arg` syntax;
- desugaring to `Type.method value arg`;
- static receiver typing and unique impl resolution;
- the absence of dynamic dispatch;
- why whitespace-only method syntax does not exist;
- the distinction between inherent methods and trait methods;
- ambiguity and missing-method diagnostics;
- interaction with field access, application precedence, and currying.

Required cross-links: Impls and methods; Methods versus trait methods;
Namespaces and name resolution; Operator precedence.

### `/docs/language/syntax/pipelines-and-low-precedence-application/` — Pipelines and `$`

Source: `docs/spec/01-syntax.md` 1.6.

The article must explain:

- `x |> f` and `f $ x` desugaring;
- left associativity of `|>` and right associativity of `$`;
- the exact precedence of `$`, including its relationship to `:=`;
- why a lone `|` is not a pipeline;
- parentheses-elimination examples and their exact expansions;
- left-then-right, exactly-once evaluation for `$`;
- why `$` does not delay evaluation or run Effects;
- why `$` does not create multi-argument calls;
- newline behavior after `$`;
- placing `if`, `match`, `do`, and lambda expressions on the right;
- why `$` cannot be overloaded, redefined, or referenced as `($)`;
- normal application type errors on a non-function left side.

Required cross-links: Function application; Layout and line continuation;
Operator precedence; Signal operators.

### `/docs/language/syntax/operator-precedence/` — Built-in operators

Source: `docs/spec/01-syntax.md` 1.7, plus semantic links to traits, Maybe, and
Signal.

The article must explain:

- the complete precedence table from access through `:=`;
- associativity at each level;
- non-associative comparison and the rejection of comparison chaining;
- short-circuit evaluation of `&&`, `||`, and `??`;
- left-to-right operand evaluation for other binary operators;
- `??` typing and fallback behavior;
- which operators can be referenced as curried function values;
- expected-type and trait-constraint resolution for overloaded operator values;
- which operators cannot become function values and why;
- the difference between unary and binary `*` and `-`;
- examples where parentheses change the parse;
- invalid chains and ambiguous operator-reference diagnostics.

Required cross-links: Standard operators and traits; Maybe; Signal read and
update operators; Custom operators.

### `/docs/language/syntax/custom-operators/` — Custom operators and fixity

Source: `docs/spec/01-syntax.md` 1.8 and the tooling fixity contract.

The article must explain:

- the complete top-level operator declaration form;
- symbol, fixity, precedence, type scheme, constraints, and body;
- desugaring and use as a curried function value;
- `infixl`, `infixr`, and non-associative `infix`;
- the allowed precedence range and why custom operators cannot outrank
  application or postfix access;
- the allowed ASCII symbol alphabet and minimum symbol length;
- every reserved symbol family and the absence of custom prefix/postfix
  operators;
- contextual `&` for requirement merge and why it is not a value operator;
- rejection of angle-bracket-only operators;
- the separate operator namespace and explicit import syntax;
- duplicate-symbol ambiguity and mixed-fixity parenthesization;
- declaration pre-scan and use before textual declaration;
- lossless raw scanning, module interface collection, flat chains, and fixity
  resolution;
- why standard overloads use traits or struct/newtype operator sugar instead.

Required examples include declaration, import, partial application, ambiguous
imports, mixed fixity, and reserved-symbol rejection.

Required cross-links: Tooling / Raw operator tokens; Header scan and module
interfaces; Flat chains and fixity resolution; Struct/newtype overloads.

### `/docs/language/syntax/reserved-words-and-names/` — Reserved words and naming

Source: `docs/spec/01-syntax.md` 1.9.

The article must explain:

- the complete reserved-word set;
- contextual foreign-block keywords;
- contextual `since` and `self` behavior;
- uppercase naming requirements for types, constructors, and traits;
- lowercase naming requirements for values, functions, fields, and aliases;
- `_` as an unreferenceable wildcard;
- case sensitivity and examples of invalid declarations;
- links from each contextual keyword to its owning syntax.

### `/docs/language/syntax/optional-record-fields/` — Optional record field syntax

Source: `docs/spec/01-syntax.md` 1.10, with semantics from the record-type and
record-pattern sections.

The article must explain:

- `field?: A` syntax and exact token placement;
- formatter normalization;
- the two contexts in which a lone `?` is meaningful;
- why `?` is not a standalone expression operator;
- where optional markers are forbidden;
- the difference between an absent optional field and a required
  `Maybe<A>`-valued field;
- field-access result types;
- optional-field subtyping and presence preservation;
- optional queries in record patterns;
- valid and invalid formatting, type, and pattern examples.

Required cross-links: Nominal and structural types; Records; Match and
patterns; TypeScript optional values.

## Syntax coverage gate

The Syntax and operators section is ready for visual design only when all eleven
leaves above are Outlined, every normative rule in specification sections
1.1–1.10 is represented, and the cross-cutting rules from function typing,
evaluation order, traits, Signal, modules, and tooling are attached to the
relevant leaf instead of being deferred to a generic “see also.”

## Language Reference / Type system

The general leaf contract applies to every route below. The scope column names
the semantic rules that must be explained, not merely mentioned.

| Route | Source | Required semantic scope |
| --- | --- | --- |
| `/docs/language/types/type-system/` | 2.1 | Static typing before execution; HM foundation; nominal ADT/struct/newtype; structural immutable records; trait constraints; `Never` coercion; why unresolved types do not become `Any` or `Unknown`. |
| `/docs/language/types/built-in-types/` | 2.2 | Exact value meanings of Int, Float, Bool, Char, String, Unit, Never; safe-Int range; negative zero; no implicit numeric conversion; Unit versus Never. |
| `/docs/language/types/type-constructors/` | 2.3 | Every built-in constructor; recursive type arguments; comma separators; right-associative function types; nested closing delimiters; independent kind and arity checking. |
| `/docs/language/types/annotations-and-inference/` | 2.4 | Required annotations for parameters, returns, and public declarations; inferred private lets; declaration placement of type parameters; module-API and recursion rationale. |
| `/docs/language/types/polymorphism/` | 2.5 | Let-polymorphism; specification-only `forall`; fresh instantiation; no ordinary value restriction; monomorphic recursive-group checking followed by generalization. |
| `/docs/language/types/nominal-and-structural-types/` | 2.6 | Nominal identity of ADT/struct/newtype; structural record identity; exact width-subtyping rule; invariant fields; absence of depth subtyping. |
| `/docs/language/types/optional-record-fields/` | 2.6 | Required versus optional presence; `Maybe` access result; distinction from required `Maybe` fields; required/optional/absent subtyping matrix; inference of record literals. |
| `/docs/language/types/closed-records/` | 2.6 | Closed as statically enumerable host requirement rather than exact-record subtyping; rejection of free variables, unresolved rows, optionals, recursive aliases, and unresolved constraints; allowance of extra provided fields. |
| `/docs/language/types/requirement-merge/` | 2.6 | `R & S` only in Effect/Stream requirement position; allowed operands; normalization and duplicate fields; conflicts and `SES-E0001`; identities and associativity; rejection as general intersection with `SES-T0501`. |
| `/docs/language/types/function-types-and-currying/` | 2.7 | Nested one-parameter function types; partial application; left-to-right binding; anonymous Unit parameter and invocation. |
| `/docs/language/types/type-identity-and-coercion/` | 2.8 | Every forbidden implicit conversion; explicit boundary conversions; only general coercions (`Never`, record width); links to Effect and Signal capability-specific coercions. |
| `/docs/language/types/recursive-declarations/` | 2.9 | Named recursion; return annotations; non-recursive let; module/block scope; local `rec` groups; monomorphic group checking; direct self-tail-call guarantee and excluded cases. |
| `/docs/language/types/kinds/` | 2.10 | `Type` and arrow kinds; `M<_>`; where constructor parameters are allowed; hole restrictions; full versus partial application; arity/kind errors without Unknown recovery. |
| `/docs/language/types/type-parameter-scope/` | 2.11 | Scope start/end; duplicate and shadowing rejection; all legal reference positions; precedence over module types; uppercase naming; no undeclared implicit type parameter. |
| `/docs/language/types/generic-functions/` | 2.12 | Quantified schemes; the five call-site inference steps; omitted and explicit type arguments; all-or-none positional arguments; return-only parameters and ambiguity. |
| `/docs/language/types/let-polymorphism-and-rank/` | 2.13 | Generalization of free variables; fresh use-site instantiation; saturated application results; retained trait constraints; environment constraints; rank-1 boundary and rejected higher-rank positions. |
| `/docs/language/types/generic-adts/` | 2.14 | Constructor schemes; fresh instantiation; resolving payload-independent parameters; pattern substitution; recursive generic ADTs. |
| `/docs/language/types/generic-structs/` | 2.15 | Field-based inference; explicit arguments; conflicting constraints; receiver substitution on access; holes; update/spread preserving exact type arguments. |
| `/docs/language/types/generic-impls-and-methods/` | 2.16 | Impl-bound parameters; required occurrence in target; method-local parameters; receiver-first impl inference followed by ordinary method inference. |
| `/docs/language/types/generic-aliases/` | 2.17 | Transparent identity; capture-avoiding substitution; arity checking in every annotation position; cycle rejection; normalization and erasure; tooling name preservation; higher-kinded alias parameters; public/private leakage checks. |
| `/docs/language/types/newtypes/` | 2.18 | Single-field nominal identity; generated constructor scheme; irrefutable constructor pattern; explicit wrap/unwrap; no `coerce` or reflection; recursion rejection; optional unused parameters; permitted backend erasure. |
| `/docs/language/types/variance/` | 2.19 | Invariance of user types and collections; independence from record width subtyping; explicit mapping; the exact Effect/Stream/Signal/Never exceptions. |
| `/docs/language/types/erasure-and-runtime-representation/` | 2.20 | Type parameters not being runtime values; no typecase/reflection/implicit specialization; backend freedom to erase or monomorphize without changing observable meaning. |

## Language Reference / Data, expressions, and patterns

| Route | Source | Required semantic scope |
| --- | --- | --- |
| `/docs/language/expressions/evaluation/` | 3.1 | Strict evaluation; application-time body evaluation; left-to-right exactly-once arguments; cold Effect exception; absence of observable reference identity. |
| `/docs/language/expressions/blocks-and-local-declarations/` | 3.2 | Legal block contents; final-expression value; declaration-only Unit; empty record versus Unit block; let scope/shadowing; local closure capture; legal local declarations. |
| `/docs/language/patterns/binding-rules/` | 3.2.1 | One shared Pattern typing model for let, do, bind, match, comprehension, and effectful for; input type, scope, refutability table; exactly-once input evaluation; simultaneous nested binding; duplicate-name and self-reference rejection; shared compiler/LSP result. |
| `/docs/language/patterns/irrefutable-patterns/` | 3.2.2 | Complete irrefutability classification for names, wildcards, tuples, records, structs, single-constructor ADTs, newtypes, arrays, and lists; mismatch diagnostics. |
| `/docs/language/expressions/conditionals/` | 3.3 | Bool-only condition; mandatory else; equal branch types; Never branch coercion; evaluation of only the selected branch. |
| `/docs/language/data/algebraic-data-types/` | 3.4 | Closed nominal variants; constructor names and types; tuple payload for multiple values; construction, visibility, matching, and generic links. |
| `/docs/language/data/structs/` | 3.5 | Nominal record-shaped data; construction and access; required fields; update/spread rules; visibility and method relationships; difference from structural records. |
| `/docs/language/data/newtypes/` | 3.6 | Expression and pattern forms; nominal boundary; explicit extraction; operator/method attachment; links to type-system representation rules. |
| `/docs/language/data/records/` | 3.7 | Structural literal/access/update/spread; required and optional fields; presence-preserving patterns; width subtyping; exactly-once field expressions; rejected duplicates and unknown fields. |
| `/docs/language/data/tuples-arrays-and-lists/` | 3.8 | Tuple arity; strict array versus persistent list; literal syntax; patterns/rest; ordering; valid conversions; indexing/head partiality via Maybe; complexity links. |
| `/docs/language/expressions/ranges-and-comprehensions/` | 3.9 | Finite Int ranges; endpoint semantics; Array/List comprehensions; generator/filter order; source evaluation; pattern mismatch filtering; result collection identity. |
| `/docs/language/patterns/match/` | 3.10 | Scrutinee evaluation; all pattern forms; nested bindings; guards; source-order arm selection; exhaustiveness/redundancy; no fallthrough; diagnostics and fixes. |
| `/docs/language/expressions/lambdas/` | 3.11 | Lambda syntax; parameter typing/inference; closure capture; currying; annotations; evaluation and relationship to named functions. |
| `/docs/language/data/impls-and-methods/` | 3.12 | Inherent impl ownership; `self` position; static lookup; visibility; duplicates; generic receiver substitution; distinction from trait instances. |
| `/docs/language/data/operator-overloads/` | 3.13 | Struct/newtype operator sugar; allowed operators; generated trait evidence or functions; ownership/coherence; operand and result typing; difference from custom operator declarations. |

## Language Reference / Traits and generic abstraction

| Route | Source | Required semantic scope |
| --- | --- | --- |
| `/docs/language/traits/model/` | 4.1 | Compile-time capability evidence and dictionary meaning; explicit rejection of inheritance and runtime-object interpretation. |
| `/docs/language/traits/declarations/` | 4.2 | Method signatures without bodies; ordinary and constructor-kinded parameters; supertraits; partial constructors; kind/arity checking. |
| `/docs/language/traits/instances/` | 4.3 | Instance syntax; distinction from impl; substituted method-type equality; instance parameters/constraints; head-occurrence rule; non-value evidence. |
| `/docs/language/traits/constraints/` | 4.4 | Where syntax; multiple constraints; unique call-site evidence; recursive conditional instance resolution; value-independent selection; concrete missing-evidence diagnostics. |
| `/docs/language/traits/method-calls/` | 4.5 | Unqualified curried use; scheme formation; concrete versus generic resolution; implicit dictionary parameters; partial application with unresolved evidence; trait-name ambiguity. |
| `/docs/language/traits/coherence/` | 4.6 | Global uniqueness within the package graph; orphan ownership rule; overlap rejection; visibility/import effects; why values never choose an instance at runtime. |
| `/docs/language/traits/standard-operators/` | 4.7 | Mapping from standard operator syntax to traits; operator functional dependencies; result-type determination; built-in short-circuit exceptions; resolution errors. |
| `/docs/language/traits/laws/` | 4.8 | Which laws are semantic library contracts; observational equality; law-test relationship; compiler does not prove arbitrary user laws. |
| `/docs/language/traits/deriving/` | 4.9 | Supported traits and nominal declarations; field/payload evidence; constructor-order semantics; generated visibility/coherence; recursive requirements; unsupported deriving diagnostics. |
| `/docs/language/traits/methods-versus-traits/` | 4.10 | Nominal inherent method lookup versus typeclass evidence; namespace, dispatch, ownership, and ambiguity differences. |
| `/docs/language/traits/do-notation/` | 4.11 | Generic Monad requirement; item forms; pure, let, and bind; no Effect-only interpretation; no implicit transformer lift. |
| `/docs/language/traits/do-desugaring/` | 4.12 | Exact nested `flatMap`/lambda expansion; scope, evaluation order, patterns, final expression, and preserved constraints. |
| `/docs/language/traits/do-block-typing/` | 4.13 | Choosing one Monad constructor; payload propagation; equality of item constructors; final type; errors for mixed monads, invalid binds, and unresolved constructor kinds. |

## Language Reference / Failure, Effect, and state

| Route | Source | Required semantic scope |
| --- | --- | --- |
| `/docs/language/effects/pure-expressions/` | 5.1 | Pure-expression boundary; values versus running computation; why I/O and mutable state cannot surface as ordinary values. |
| `/docs/language/effects/maybe/` | 5.2 | Absence as data; constructors and operations; `??` type, right associativity, laziness, and short-circuiting; distinction from nullable boundaries and failure. |
| `/docs/language/effects/either/` | 5.3 | Synchronous typed success/failure; constructor roles; mapping; first-failure behavior; distinction from Effect and accumulating Validation. |
| `/docs/language/effects/effect-type/` | 5.4 | Cold `Effect<R,E,A>`; meaning of R, E, A; construction versus execution; Functor/Applicative/Monad behavior and left-to-right composition. |
| `/docs/language/effects/effect-functions-contract-form/` | 5.4 | `effect fn` contract syntax; required `->`; `with`/`fails` defaults; normalization to a function returning Effect; body-contract checking; canonical service-name expansion. |
| `/docs/language/effects/effect-functions-inferred-form/` | 5.4 | Compact form without signature clauses; inference of normalized R/E/A; exported interface requirements; ambiguity handling; why unrelated failures are not auto-unioned. |
| `/docs/language/effects/effectful-for/` | 5.4 | Irrefutable pattern; Iterable order; Effect body/result; forEach desugaring; empty and first-failure behavior; no auto-parallelism, break, or continue; forEachUntil alternative. |
| `/docs/language/effects/environment-requirements/` | 5.5 | Required structural record; union of service fields; matching duplicate types; optional-field rejection; requirement widening; provide/service; no globals; compile-time requirement merge. |
| `/docs/language/effects/error-channels/` | 5.6 | Recoverable E; explicit mapError to application ADTs; no synthesized union; only Never widening; recover excluding defect and cancellation. |
| `/docs/language/effects/task/` | 5.7 | Exact transparent alias; no separate identity/runtime/ABI; shared instances; environment-free fallible Effects still written explicitly. |
| `/docs/language/effects/sequential-and-parallel/` | 5.8 | Sequential flatMap/operator/do; explicit parallel; result/failure/cancellation order; prohibition on compiler auto-parallelization. |
| `/docs/language/effects/runtime-boundaries/` | 5.9 | No general runner in ordinary code; allowed host boundaries; one environment assembly; target-capability validation before execution; no runtime missing-service lookup. |
| `/docs/language/effects/defects/` | 5.10 | Defect versus typed failure; canonical examples; no implicit conversion or ordinary catch; design rule against classifying expected external failure as defect. |
| `/docs/language/effects/cancellation-and-resources/` | 5.11 | Cooperative cancellation; child propagation; bracket/acquireRelease; all terminal outcomes; atomic acquire handoff; shared root/lexical scope rules; idempotent close; masked finalizers; LIFO; defect priority; EffectExit. |
| `/docs/language/effects/scheduler-fairness/` | 5.11 | FIFO runnable queue; weak fairness; checkpoint list; no preemption of pure work; deterministic queueing constraints; scoped versus sibling finalizer ordering. |
| `/docs/language/effects/fiber-supervision/` | 5.11 | Scope ownership; no daemon fiber; shutdown cancellation; child cleanup before close; unobserved failure; join/await/interrupt outcomes and cancellation behavior. |
| `/docs/language/effects/signals-and-transactions/` | 5.12 | Signal/MutableSignal types and one-way capability coercion; Effectful state access; opaque SignalChange; set/update transactions; staged order; atomic commit/rollback on defect or pre-commit cancellation. |
| `/docs/language/effects/derived-signals/` | 5.13 | Pure constructors; explicit dependency graph; topological at-most-once recomputation; glitch freedom; explicit distinct; Functor/Applicative; switchMap lifetime and transaction semantics. |
| `/docs/language/effects/signal-operators/` | 5.14 | `*source` and `target := value` parse/desugar/type; snapshot as Task; distinction from multiplication; MutableSignal-only set; no variable/field assignment; atomic update for read-modify-write. |
| `/docs/language/effects/subscriptions-and-lifetime/` | 5.15 | Initial notification; registration order; no overlapping observer call; queued reentrant updates; Never failure requirement; defect handling; opaque/idempotent Subscription; scope cleanup; derived dependency release; no Monad instance. |
| `/docs/language/effects/exceptions-and-algebraic-effects/` | 5.16 | Absence of throw/catch/try and perform/handle; foreign exception conversion; why one Effect model owns environment, errors, resources, and cancellation. |

## Language Reference / Modules

| Route | Source | Required semantic scope |
| --- | --- | --- |
| `/docs/language/modules/identity/` | 6.1 | One file per module; canonical identity from package and normalized path; spelling versus identity; duplicate identity and path normalization. |
| `/docs/language/modules/packages/` | 6.2 | Package boundary and manifest identity; how module identity incorporates package identity; local and dependency packages. |
| `/docs/language/modules/top-level/` | 6.3 | All legal top-level declarations; declaration ordering and namespace effects; what is unavailable at top level. |
| `/docs/language/modules/visibility/` | 6.4 | Private-by-default declarations; `pub`; public constructors/fields/methods; opaque exposure; public-signature leakage diagnostics. |
| `/docs/language/modules/imports/` | 6.5 | Named, aliased, namespace, and operator imports; explicit visibility; collisions; local binding relationship; imported instance visibility. |
| `/docs/language/modules/specifier-resolution/` | 6.6 | Relative, package, and standard specifiers; manifest/export-map lookup; canonical candidates; zero/multiple-candidate errors; path and package identity. |
| `/docs/language/modules/re-exports/` | 6.7 | Named and namespace re-export; preserved declaration identity; public API addition; graph participation. |
| `/docs/language/modules/namespaces-and-resolution/` | 6.8 | Five separate namespaces; duplicate rules; exact unqualified lookup order; same-stage ambiguity; imports never silently retarget an ambiguous name. |
| `/docs/language/modules/dependency-graphs/` | 6.9 | Cycles forbidden for values, types, and traits; complete cycle diagnostics; no type-only exception; moving shared types; local rec alternative. |
| `/docs/language/modules/initialization/` | 6.10 | Dependency topological initialization once; source-order top-level lets; declaration availability; function runtime bindings; delayed versus immediate uninitialized access; `SES-N0201`; foreign pure-load/task-load restrictions. |
| `/docs/language/modules/entry-points/` | 6.11 | Manifest-selected single public main; anonymous Unit parameter; closed environment; providing application services; concrete failure with coherent Show; Unit success; host exit-status ownership; no direct exit primitive; libraries and imported mains. |

## Core language coverage gate

The core Language Reference is not ready for mockup approval until Syntax,
Types, Data and expressions, Traits, Effects and signals, and Modules all have
one leaf per row above, plus explicit coverage back-links to every rule in
specification chapters 0–6. Chapter-level landing pages contain no semantic
rules that exist only on the landing page.

## Interop Reference / TypeScript

| Route | Source | Required semantic scope |
| --- | --- | --- |
| `/docs/interop/typescript/boundary-model/` | 7.1 | Interop as an explicit calling and conversion boundary, not adoption of the TypeScript type system; visible nullish values, throws, rejection, overload choice, and Seseragi currying. |
| `/docs/interop/typescript/foreign-modules/` | 7.2 | Private and public foreign blocks; resolver and exact host-module identity; pure-load versus task-load selection across all blocks; initialization guarantees; lazy single-flight load and memoized failure; exact property keys; nested namespaces and own-property lookup; receiver rules. |
| `/docs/interop/typescript/pure-calls/` | 7.3 | Required synchronous, deterministic, non-throwing, state-independent contract; safe conversion obligation; Promise rejection as a declaration defect. |
| `/docs/interop/typescript/task-calls/` | 7.3 | Cold invocation; automatic Effect shape; synchronous throw and Promise rejection capture; module-load and lookup phases; metadata-preserving `Js.Error`. |
| `/docs/interop/typescript/arity-and-currying/` | 7.4 | Seseragi-side partial application without host invocation; one uncurried host call only after all arguments arrive. |
| `/docs/interop/typescript/boundary-types/` | 7.5 | Meaning and legal operations of every `Js.*` carrier; raw identity/mutability; no implicit conversion to normal types. |
| `/docs/interop/typescript/primitive-conversion/` | 7.6 | Bool/Char/String/Float/Int/BigInt/Unit/Never/Bytes projection; Unicode validation; safe-integer validation and `-0`; explicit numeric choice; byte copying. |
| `/docs/interop/typescript/collections-and-records/` | 7.7 | Snapshot/copy rules for arrays, tuples, records, optionals, and bytes; no automatic List or nominal data conversion; presence preservation; no mutable host sharing. |
| `/docs/interop/typescript/optional-nullable-and-rest/` | 7.8 | Raw optional parameters as `Js.UndefinedOr`; undefined versus omission; hand-written Maybe adapters; nullable conversion; arity-specific omission; one trailing rest parameter and spread call. |
| `/docs/interop/typescript/overloads/` | 7.9 | One local binding per explicitly selected host overload; no union dispatch and no delegation to the TS checker at runtime. |
| `/docs/interop/typescript/classes-and-members/` | 7.10 | Opaque class types; constructor, method, and property call conventions; task/pure rules; receiver and getter behavior; host identity without public identity comparison. |
| `/docs/interop/typescript/callbacks/` | 7.11 | The narrow auto-adapted synchronous pure callback case; retained, repeated, asynchronous, and Effectful callbacks requiring `Js.Callback`, resource APIs, and explicit lifetime/cancellation contracts. |
| `/docs/interop/typescript/exporting-seseragi/` | 7.12 | Uncurried wrappers; exact projection for primitives, records, nominal data, ADTs, Maybe/Either, Task/Effect, and opaque types; Int validation; prohibition on exporting unprovided environments. |
| `/docs/interop/typescript/abi-stability/` | 7.13 | Wrapper and declaration as the public ABI; internal closure/dictionary/layout freedom; exact signature, order, opacity, and variant changes that break ABI. |
| `/docs/interop/typescript/generic-public-abi/` | 7.14 | Exportable unconstrained rank-1 generics; opaque leaf pass-through and outer-shape checking; non-exportable generic forms; explicit trait dictionary configuration and metadata; generic nominal data branding. |
| `/docs/interop/typescript/source-maps-and-stacks/` | 7.15 | Source map v3, embedded source, package URIs, generated markers, cross-language frame labeling, causal Initiated/Thrown/Observed groups, missing-map behavior, path privacy, and RuntimeDiagnostic JSON schema. |

## Interop Reference / `.d.ts` conversion

| Route | Source | Required semantic scope |
| --- | --- | --- |
| `/docs/interop/bindings/converter-model/` | 8.1 | Generation of foreign-binding candidates rather than translation of the TS type system; semantic-preservation rule; no silent Any weakening; explicit boundary carriers; no inferred purity; mandatory report. |
| `/docs/interop/bindings/inputs-and-symbols/` | 8.2 | TypeScript program/symbol-table inputs; post-resolution imports, re-exports, merging, and augmentation; entry exports only; opaque transitives; explicit global selection. |
| `/docs/interop/bindings/generated-outputs/` | 8.3 | Binding module, provenance metadata, report, and manual-work list; deterministic sort/output; generated-root placement; separate adapters; public raw block but no direct application re-export. |
| `/docs/interop/bindings/primitives/` | 8.4 | Complete primitive conversion matrix; contextual void; any rejection/unsafe warning; explicit Char and Int overrides with runtime checks; bigint behavior. |
| `/docs/interop/bindings/nullability-and-optionals/` | 8.5 | Exact nested raw wrappers for null/undefined; optional parameter/property handling; why Maybe conversion belongs in adapters. |
| `/docs/interop/bindings/collections-tuples-and-objects/` | 8.6 | Readonly versus mutable arrays; tuples; Uint8Array copying; non-equivalent binary types; record conversion opt-in; opaque mutable/indexed objects. |
| `/docs/interop/bindings/functions/` | 8.7 | Curried candidate shape; task-by-default; pure approval; Promise success extraction; throw/rejection; callback carrier default and approved normal-function exception. |
| `/docs/interop/bindings/overloads/` | 8.8 | Independent candidate report; configured signature and local name; no union function; error on unselected set; no implementation-signature leakage. |
| `/docs/interop/bindings/generics/` | 8.9 | Supported first-order subset; preserved parameter order/arity/positions; recorded default expansion; explicit `extends`-to-trait mappings only. |
| `/docs/interop/bindings/classes-and-enums/` | 8.10 | Opaque class and public member candidates; private/protected omission; first-order generic classes; enum representation and unsupported forms. |
| `/docs/interop/bindings/unions-and-intersections/` | 8.11 | Special nullish unions; refusal of arbitrary unions/intersections; explicit adapters or supported discriminated-union opt-in; no requirement-merge abuse. |
| `/docs/interop/bindings/unsupported-types/` | 8.12 | Complete unsupported TS feature list; error versus opaque/report behavior; no fallback to normal Seseragi types. |
| `/docs/interop/bindings/discriminated-unions/` | 8.13 | Opt-in preconditions; stable discriminator; variant/payload mapping; naming/collision rules; rejected ambiguous/open cases. |
| `/docs/interop/bindings/updating-declarations/` | 8.14 | Deterministic regeneration; provenance diff; stale generated inputs; adapter isolation; breaking-change reporting. |
| `/docs/interop/bindings/configuration/` | 8.15 | Configuration schema, exact symbol identities, mode and type overrides, overload selection, record/union opt-ins, validation, and digesting. |
| `/docs/interop/bindings/callback-lifetimes/` | 8.16 | Configured sync/once/multi-shot/retained/asynchronous behavior; release and cancellation ownership; unsafe or incomplete configuration rejection. |
| `/docs/interop/bindings/generated-names/` | 8.17 | Stable sanitization, casing, qualification, overload suffixes, reserved words, and deterministic collision handling. |
| `/docs/interop/bindings/declaration-merging/` | 8.18 | Resolved merged symbol treatment; nested namespace export generation; value/type sides; conflicts and deterministic ordering. |

## Projects Reference

| Route | Source | Required semantic scope |
| --- | --- | --- |
| `/docs/projects/manifest/` | 11.1 | Complete `seseragi.toml` schema, defaults, validation, target/tool tables, paths, and unknown-field behavior. |
| `/docs/projects/package-identity-and-version/` | 11.2 | Stable package identity, version semantics, dependency identity, source provenance, and conflict rules. |
| `/docs/projects/layout/` | 11.3 | Canonical roots and file roles; source/test/generated/public/output separation; accepted alternatives only through manifest configuration. |
| `/docs/projects/module-paths/` | 11.4 | File-to-module mapping, normalization, invalid paths, identity, and case/collision behavior. |
| `/docs/projects/export-maps/` | 11.5 | Public subpath contract, private modules, resolution, validation, and package-consumer behavior. |
| `/docs/projects/dependencies/` | 11.6 | Dependency declarations, version/source choice, aliasing, graph resolution, duplicate identity, and lock interaction. |
| `/docs/projects/imports-and-tests/` | 11.7 | Package-internal imports, test visibility and roots, test module identity, and production-boundary separation. |
| `/docs/projects/generated-bindings/` | 11.8 | Generated roots, provenance, import paths, freshness, and separation from authored source/public adapters. |
| `/docs/projects/foreign-host-inputs/` | 11.9 | Host dependency/configuration inputs, resolver ownership, deterministic build metadata, and target boundaries. |
| `/docs/projects/executable-entry-points/` | 11.10 | Manifest entry selection, main contract, target selection, output kinds, execution preparation, and failure reporting. |
| `/docs/projects/web-documents-and-assets/` | 11.10 | Web document inputs, public assets, safe normalized paths, copying/build ownership, and output collision rules. |
| `/docs/projects/lockfiles/` | 11.11 | Locked identities, dependencies, generated inputs, providers, targets, freshness, explicit update, locked-mode failure, and reproducibility. |
| `/docs/projects/workspace-discovery/` | 11.12 | Project/root discovery, nested workspaces, invocation location, and deterministic selected root. |
| `/docs/projects/logical-inputs-and-loaders/` | 11.13 | Logical project files, loader-adapter contract, normalized paths/bytes, source identity, host independence, and diagnostics. |

## Tooling Reference

| Route | Source | Required semantic scope |
| --- | --- | --- |
| `/docs/tooling/contracts/model/` | 12.1 | Why parser, formatter, compiler, LSP, document generator, and Playground share contracts; authoritative inputs and prohibited divergent frontends. |
| `/docs/tooling/contracts/syntax-pipeline/` | 12.2 | One shared token/CST pipeline across compiler, formatter, LSP, highlighter, and Playground; losslessness and identity requirements. |
| `/docs/tooling/contracts/raw-operator-tokens/` | 12.3 | Operator token preservation before fixity knowledge; boundaries, invalid sequences, and downstream ownership. |
| `/docs/tooling/contracts/header-scan/` | 12.4 | Header extraction, module interfaces, operator declarations/imports, dependency data, and no body evaluation. |
| `/docs/tooling/contracts/fixity-resolution/` | 12.5 | Flat chains, imported/local fixity, deterministic regrouping, ambiguity, parentheses, and incremental invalidation. |
| `/docs/tooling/contracts/incomplete-source/` | 12.6 | Recovery nodes/holes, retained tokens and symbol scopes, error ranges, no recovery type leaking to public interface/runtime. |
| `/docs/tooling/language-server/` | 12.7 | Shared analysis inputs; hover, completion, definition, references, rename, diagnostics, code actions, revisions, and cross-module identity. |
| `/docs/tooling/formatter/` | 12.8 | Canonical layout; idempotence; CST/token preservation; comments, escapes, operator chains, incomplete source, range formatting, and fixed style policy. |
| `/docs/tooling/contracts/conformance/` | 12.9 | Minimum shared cases and artifact agreement across parser, formatter, compiler, LSP, and highlighting. |
| `/docs/tooling/syntax-highlighting/` | 12.10 | Lexical/semantic token boundaries, contextual names, modifiers, incomplete input, and token agreement. |
| `/docs/tooling/playground/overview/` | 12.11 | Same frontend/semantics as CLI, browser adapter boundaries, no parallel language implementation, execution and diagnostics. |
| `/docs/tooling/contracts/minimum-language-surface/` | 12.12 | Required parser/semantic/tooling surface and refusal to claim unsupported contract-only features as current. |
| `/docs/tooling/contracts/analysis-api/` | 12.12.1 | Versioned shared analysis schema, stable identities, ranges, revision/source association, and consumer behavior. |
| `/docs/tooling/contracts/type-documents/` | 12.12.2 | Typed document contents, symbol/use relationships, inferred types, constraints, recovery, and deterministic serialization. |
| `/docs/tooling/contracts/type-display/` | 12.12.3 | Canonical display of aliases, normalized types, constraints, effects, records, kinds, and unresolved recovery state. |
| `/docs/tooling/diagnostics/` | 12.13 | Stable codes/severity, primary and related ranges, labels, notes, fixes, text/JSON forms, sorting, deduplication, and source-version ownership. |
| `/docs/tooling/type-inference-explanations/` | 12.14 | User-requested explanation graph, constraints/evidence/expected types, stable presentation, and no change to inference. |
| `/docs/tooling/exhaustive-match-fixes/` | 12.15 | Missing-pattern computation, insertion location/format, wildcard behavior, preserved guards/comments, and `todo` safety. |
| `/docs/tooling/documentation-comments/` | 12.16 | Comment syntax/attachment, supported markup, parameter/type links, inherited/generated API docs, escaping, and unresolved link diagnostics. |
| `/docs/tooling/test-runner/` | 12.17 | Test discovery, isolation, Effect environment, ordering, deterministic services, output/results, cancellation, and exit behavior. |
| `/docs/tooling/deprecations/` | 12.18 | Metadata syntax, since/replacement, diagnostics, references/completion display, generated API metadata, and version stability. |
| `/docs/tooling/options-and-target-capabilities/` | 12.19 | Versioned option schema; stable/experimental options; exact range units; doc output path; target capability query schema; deterministic ordering; exit codes; no host probing or compile-time reflection. |
| `/docs/tooling/learning-surface-coverage/` | 12.20 | Playground/Tour/Discover roles; starter/blank/reset; routing new surfaces to Tour, Recipe, Showcase, Reference, or Deep Dive; current versus excluded; separate fixture and teaching coverage. |
| `/docs/tooling/playground/virtual-workspaces/` | 12.21 | Full workspace state; normalized paths; immutable atomic operations; folder rename/delete propagation; entry/active fallback. |
| `/docs/tooling/playground/project-compiler/` | 12.22 | Schema-1 virtual package; shared native/WASM project graph; generated modules; diagnostics; analysis; single-file compatibility. |
| `/docs/tooling/playground/spec-fixtures/` | 12.23 | Phase versus availability; current evidence; responsible runner; negative diagnostic exactness; repository exclusion versus standalone semantics. |
| `/docs/tooling/playground/explorer/` | 12.24 | Tree ordering/state/actions; validation and atomic delete; desktop/mobile behavior; complete keyboard/accessibility contract; resize persistence. |
| `/docs/tooling/playground/editor-tabs/` | 12.25 | Open/active/dirty model; rename/delete propagation; one editor with per-file state/history/scroll; stale analysis rejection. |
| `/docs/tooling/playground/workspace-execution/` | 12.26 | Full revision request; entry build/staging; relative modules; service adapters; diagnostics navigation; stale result rejection; project reset semantics. |
| `/docs/tooling/playground/html-preview/` | 12.27 | Native/fallback fullscreen; safe areas, focus/scroll/state preservation; iframe sandbox/CSP; stable image examples; responsive Tour navigation accessibility. |

## Application Reference / Web UI

| Route | Source | Required semantic scope |
| --- | --- | --- |
| `/docs/applications/web/module-boundaries/` | 13.1 | Pure tree, SSR, DOM adapter, events, Signals, target namespaces, and ownership between modules/backends. |
| `/docs/applications/web/html-and-children/` | 13.2 | `Html<Action>` meaning; text/elements/fragments; child normalization/order; no host-node identity; valid child types and rendering. |
| `/docs/applications/web/props/` | 13.3 | Typed common/tag-specific props; optional presence; class/style/data/ARIA projection; form/control properties; event/ref/key exclusions and validation. |
| `/docs/applications/web/events/` | 13.4 | Immutable event snapshots; `EventAction`; synchronous browser control versus queued Action; delegation; event ordering; failure/cancellation and listener lifetime. |
| `/docs/applications/web/ime-composition/` | 13.4.1 | Composition lifecycle; deferred controlled-value writes; committed input; event ordering; focus/selection preservation. |
| `/docs/applications/web/safe-output/` | 13.5 | Closed safe tags/attributes/style/URL APIs; escaping once; forbidden injection forms; URL validation; raw escape hatches and trust boundaries. |
| `/docs/applications/web/pure-tree-semantics/` | 13.6 | Structural immutable tree meaning; equality/identity limitations; deterministic rendering; no component lifecycle hidden in pure functions. |
| `/docs/applications/web/state-ownership/` | 13.6.1 | Feature-owned Signal/resource state; pure component boundary; event-to-update flow; no hook order/global virtual tree/hidden component identity. |
| `/docs/applications/web/server-rendering/` | 13.7 | Deterministic HTML serialization, escaping, void elements, attributes, event omission, links/security, document behavior, and target independence. |
| `/docs/applications/web/dom-services-and-targets/` | 13.8 | Dom capability and closed targets; mount modes/content/mount handles; exact target resolution; typed failures; cleanup ownership. |
| `/docs/applications/web/event-lifetime/` | 13.9 | Dispatch queue, ordering, reentrancy, handler lookup, Action execution, failure shutdown, delegated listeners, unmount/cancellation cleanup. |
| `/docs/applications/web/signal-bindings/` | 13.10 | Typed BindingTarget; static/leaf/region update units; exact resolution; sink ownership; equality skips; controlled inputs; keyed reconciliation; transaction stability; scope cleanup; trace schema. |
| `/docs/applications/web/large-scenes/` | 13.10 | Splitting high-frequency typed leaf targets from keyed structural regions; projections/distinct; stable refs; camera/scene examples; why no generic binding HKT is added. |
| `/docs/applications/web/hydration/` | 13.11 | Fresh, strict, and replace modes; exact match dimensions; text merging; mismatch mutation guarantees; delayed interactivity; latest Signal snapshot attachment. |
| `/docs/applications/web/targets-and-interop/` | 13.12 | Backend-independent Html; opaque ABI; adapter-only host DOM; shared SSR/test/browser semantics; in-memory and multi-engine conformance; deterministic DOM fixture schema. |
| `/docs/applications/web/document-metadata/` | 13.12 | Typed meta props; projection order/names; once-only escaping; omission; protected attributes; pure metadata composition. |
| `/docs/applications/web/svg-and-browser-interaction/` | 13.13 | Separate SVG namespace/tree; explicit conversion; typed reactive targets; finite numeric validation; pointer/wheel snapshots; synchronous capture controls; Effectful DOM measurements/observers; cancellation and cleanup. |

## Internals Reference / Performance

| Route | Source | Required semantic scope |
| --- | --- | --- |
| `/docs/internals/performance/model/` | 14.1 | Purpose, readers, relationship to semantics/library contracts, and how guarantees constrain backends. |
| `/docs/internals/performance/guarantee-kinds/` | 14.2 | Semantic, complexity, and profile guarantees; what each kind promises and what measurements do not make normative. |
| `/docs/internals/performance/as-if-rule/` | 14.3 | Observable values, order, failures, effects, cancellation, resources, interop, and diagnostics that optimization cannot change. |
| `/docs/internals/performance/cost-classes/` | 14.4 | Allocation, traversal, lookup, copy, suspension, and host-boundary cost vocabulary used across docs. |
| `/docs/internals/performance/erased-abstractions/` | 14.5 | Newtype/sugar/type-argument erasure and conditions preserving visibility, traits, patterns, and source mapping. |
| `/docs/internals/performance/functions-currying-and-traits/` | 14.6 | Closure/partial-application costs; dictionary passing; specialization freedom; observable constraints. |
| `/docs/internals/performance/data-collections-and-fusion/` | 14.7 | Data representation freedom; Array/List/Map/Set costs; fusion conditions; preserved strictness, order, short-circuit, and allocation observability. |
| `/docs/internals/performance/recursion-and-stack-safety/` | 14.8 | Guaranteed direct self-tail-call behavior; excluded mutual/non-tail cases; stack-safe library/runtime chains. |
| `/docs/internals/performance/effect-stream-signal/` | 14.9 | Cold construction, suspension and scheduling costs; stack safety; Stream demand/buffers; Signal graph/transaction complexity. |
| `/docs/internals/performance/html-and-dom/` | 14.10 | Pure tree/SSR costs; leaf versus region updates; keyed bounded reconciliation; listener/subscription ownership; prohibited hidden global work. |
| `/docs/internals/performance/build-profiles/` | 14.11 | Development/release semantic equivalence; allowed optimization/debug differences; deterministic inputs. |
| `/docs/internals/performance/verification-and-benchmarks/` | 14.12 | Benchmark ownership, warmup/input/output, regression evidence, semantic gates, and non-normative measurements. |
| `/docs/internals/production/artifact-manifest/` | 14.13 | Manifest schema, identities/digests/sizes/source maps, deterministic ordering, and provenance. |
| `/docs/internals/production/reachability/` | 14.14 | Entrypoints and graph roots; public/exported/reflective boundaries; tree shaking and retained side effects. |
| `/docs/internals/production/runtime-retention/` | 14.15 | Which official runtime parts remain for used language surfaces and why; no accidental whole-runtime retention. |
| `/docs/internals/production/bundle-layout/` | 14.16 | Output files/chunks/assets, stable relationships, target rules, and loader ownership. |
| `/docs/internals/production/minification-and-source-maps/` | 14.17 | Release minification, name/source preservation, map policy, path privacy, and diagnostic use. |
| `/docs/internals/production/regression-gates/` | 14.18 | Artifact size/content/source-map comparisons; allowed baseline updates; deterministic release evidence. |

## Standard Library Reference / Semantic contracts

These pages explain library-wide meaning. They do not replace module or symbol
pages.

| Route | Source | Required semantic scope |
| --- | --- | --- |
| `/docs/library/contracts/prelude/` | 9.1 | Complete auto-import surface; constructor and trait-method availability; shadowing rules; `todo` type, build-stopping diagnostic, and collision-free code-action behavior. |
| `/docs/library/contracts/maybe-and-either/` | 9.2 | Typeclass operations versus type-specific operations; lazy Maybe fallback; instance semantics; Either first failure versus Validation accumulation. |
| `/docs/library/contracts/collections/` | 9.3 | Array/List representation and core operations; reduce direction and mandatory initial value; total API policy; explicit conversions; structural Eq/Ord/Hash evidence and nominal ownership. |
| `/docs/library/contracts/equality-ordering-and-hashing/` | 9.4 | Eq/Ord/Hash signatures and laws; Float exclusions; exact standard instances and ordering rules; hash consistency. |
| `/docs/library/contracts/semigroup-and-monoid/` | 9.5 | Laws; Unit-parameter methods; Zero/One distinction; aggregate function semantics and short-circuiting; standard instances; Sum/Product wrappers; absence of special monoid syntax. |
| `/docs/library/contracts/numeric-operations/` | 9.6 | Int and BigInt overflow/division/remainder/exponent behavior; checked APIs; Float IEEE behavior and missing canonical equality/order; built-in Bool logic. |
| `/docs/library/contracts/functor-applicative-and-monad/` | 9.7 | Exact standard instance set; per-type apply/flatMap order and failure; Validation/Signal exclusions; law set and Effect observational equality. |
| `/docs/library/contracts/effect/` | 9.8 | Core constructors/error/provision/parallel/traversal signatures; cold defer; requirement widening; deterministic parallel failure/result/cleanup; sequential and short-circuit traversal. |
| `/docs/library/contracts/monad-transformers/` | 9.9 | Nominal transformer representations/modules; run/lift/special operations; evidence requirements; no implicit lifting; order, state, writer combination, and Effect interaction. |
| `/docs/library/contracts/resources-and-signals/` | 9.10 | How standard resource and Signal APIs realize chapter-5 cancellation, scope, transaction, and lifetime rules. |
| `/docs/library/contracts/show-and-debug/` | 9.11 | Pure Show versus developer Debug; totality and determinism; structural and nominal instances; cycles/limits; template interpolation; no I/O. |
| `/docs/library/contracts/console/` | 9.12 | Console service requirement, print/println/value rendering, typed host failure, ordering, newline/encoding, and target behavior. |
| `/docs/library/contracts/structured-logging/` | 9.13 | Logger service; levels, fields, spans/context, deterministic value conversion, failure/backpressure, and separation from console output. |

## Standard Library Reference / Module surface

Each route below has a hand-authored semantic introduction and generated child
pages for every compiler-owned public type, constructor, trait, function,
instance, deprecation, and source identity.

| Route family | Source | Required semantic scope beyond generated signatures |
| --- | --- | --- |
| `/docs/library/overview/` | 10.1 | Surface policy, module ownership, stability, portability, and relationship to semantic contracts. |
| `/docs/library/common-api-rules/` | 10.2 | Registry ownership; naming/currying/order/totality rules; capability API versus Provider boundary; public-surface compatibility. |
| `/docs/library/prelude/` | 10.3 | Exact exports and collision/shadow behavior, with links to their owning modules. |
| `/docs/library/maybe/`, `/docs/library/either/`, `/docs/library/validation/` | 10.4 | Constructor and conversion semantics; map/apply/flatMap; short-circuit versus accumulation; traversal and error combination. |
| `/docs/library/collection/` | 10.6 | Iterable/Reducible/Traversable contracts; iteration order; reduction and traversal laws; early termination. |
| `/docs/library/array/` | 10.5 | Strict contiguous representation; indexing/update/copy/slice/sort; total APIs; order and complexity; conversions. |
| `/docs/library/list/` | 10.5 | Persistent linked representation; cons/head/tail/append/reverse; pattern relationship; order and complexity; conversions. |
| `/docs/library/non-empty-list/` | 10.5 | Non-empty invariant, construction/conversion, folds, instances, and operations preserving or losing the invariant. |
| `/docs/library/map/`, `/docs/library/set/` | 10.5 | Key evidence; persistent operations; lookup/update/delete/traversal order; collision behavior; complexity. |
| `/docs/library/map/hash-seed/` | 10.5 | Process/runtime seed ownership; observable iteration restrictions; deterministic serialization rules; test behavior and security boundary. |
| `/docs/library/text/` | 10.7 | Scalar-based String operations, slicing/searching/normalization boundaries, builders, totality, and complexity. |
| `/docs/library/regex/` | 10.7 | Pattern compilation, flags, match captures/ranges, typed errors, Unicode behavior, replacement, and engine portability limits. |
| `/docs/library/text/grapheme/` | 10.8 | Grapheme segmentation version/data, iteration and slicing, cost, and distinction from Char/scalar/byte. |
| `/docs/library/text/unicode/` | 10.8 | Unicode properties/normalization/case operations, versioning, deterministic tables, and invalid scalar impossibility. |
| `/docs/library/number/`, `/docs/library/int/`, `/docs/library/float/`, `/docs/library/math/` | 10.8 | Parsing/formatting/conversion/checking; safe Int; IEEE special values/total comparison; domain errors; deterministic math boundary. |
| `/docs/library/big-int/`, `/docs/library/decimal/` | 10.8 | Arbitrary precision and decimal model; parsing/formatting; rounding/context; checked failure; interop and performance costs. |
| `/docs/library/bytes/`, `/docs/library/bytes/hex/`, `/docs/library/bytes/base64/` | 10.8 | Immutable bytes; slicing/copying/equality; encoding/decoding variants and typed errors; interop snapshot rules. |
| `/docs/library/json/` | 10.9 | JSON value model; parser/encoder determinism; decoder composition/path errors; numeric/string restrictions; JsonEncode/JsonDecode. |
| `/docs/library/time/`, `/docs/library/clock/` | 10.10 | Instant/duration/date-time meanings; parsing/formatting/time zones; monotonic versus wall clock; Effect service ownership. |
| `/docs/library/random/`, `/docs/library/entropy/` | 10.10 | Pseudo-random versus secure entropy; deterministic test providers; ranges/distributions; bias avoidance; capability requirements. |
| `/docs/library/effect/` | 10.11 | Full module surface for construction, transformation, errors, requirements, temporal control, resource scopes, concurrency, Fiber, traversal, and cancellation. |
| `/docs/library/ref/`, `/docs/library/deferred/`, `/docs/library/queue/`, `/docs/library/semaphore/` | 10.11 | Atomic state or synchronization invariant; creation/use Effects; waiter ordering, fairness, cancellation, shutdown, and resource behavior. |
| `/docs/library/stream/` | 10.12 | Cold repeated execution; element/failure/end model; sequential composition; resource scope; demand, buffers, overflow, time operators, terminal operations, and Signal conversion. |
| `/docs/library/signal/` | 10.13 | Source/derived construction; atomic transactions; subscription/resource operations; graph semantics and explicit Stream bridges. |
| `/docs/library/stdin/`, `/docs/library/console/`, `/docs/library/logger/` | 10.14 | Service requirements; encoding/order/failure; line/stream behavior; structured versus presentation output. |
| `/docs/library/filesystem/` | 10.14 | Path model; files/directories/metadata; byte/text streams; atomic replace; symlinks; resource handles; target differences and errors. |
| `/docs/library/process/` | 10.14 | Process information, environment, signals, graceful shutdown, child process creation/I/O/status, cancellation and cleanup; no arbitrary direct exit. |
| `/docs/library/http/` | 10.15 | Request/response types; headers/body streams; client Effects; redirects/timeouts/cancellation; typed transport/protocol failures. |
| `/docs/library/http/server/` | 10.15 | Server resources; effectful handler bridge; request body lifetime; response streaming; concurrency, cancellation, shutdown, and target capability. |
| `/docs/library/web/file/`, `/docs/library/http/multipart/` | 10.15 | File values/streams; multipart boundaries/parts/limits; upload lifetime; decoding and cleanup failures. |
| `/docs/library/sse/` | 10.15 | Event encoding/parsing; reconnect metadata; client/server Stream behavior; cancellation and resource ownership. |
| `/docs/library/websocket/`, `/docs/library/websocket/server/` | 10.15 | Connection resource; frame/message model; send/receive ordering; close handshake; errors/cancellation/backpressure; server upgrade. |
| `/docs/library/web/navigation/` | 10.16 | Location/history snapshots and Effects; URL validation; event streams; browser-only capability and target rejection. |
| `/docs/library/web/storage/` | 10.17 | Storage areas, string/JSON adapters, quota/security failures, change events, target ownership, and explicit Effect boundary. |
| `/docs/library/test/` | 10.18 | Test declarations/assertions; equality/diff; Effect testing; deterministic services; resource leaks; law tests and runner integration. |
| `/docs/library/benchmark/` | 10.19 | Benchmark declaration/configuration; warmup/sampling; Effect boundary; result schema; non-normative timing and regression use. |
| `/docs/library/adapters/` | 10.20 | Optional adapter discovery/import; core exclusion; target/provider requirements; versioning and failure to silently alter standard semantics. |

## Internals Reference / Runtime Provider Contract

Every section below is its own route. Rejection-condition pages remain separate
checklists; they are not hidden inside adjacent articles.

| Route | Source | Required semantic scope |
| --- | --- | --- |
| `/docs/internals/providers/model/` | 15.1 | Contract purpose, portability boundary, authoritative schema/spec relationship, and separation from backend ABI. |
| `/docs/internals/providers/services-and-requirements/` | 15.2 | Service identity, field requirements, canonical names, structural records, and prohibition on exposing provider identity to applications. |
| `/docs/internals/providers/contract-version/` | 15.3 | Version fields, negotiation scope, compatibility, and independent roles. |
| `/docs/internals/providers/logical-types/` | 15.4 | Backend-neutral logical values, supported shapes, normalization, and rejection of backend-native leakage. |
| `/docs/internals/providers/operation-identity/` | 15.5 | Stable service/operation keys, operation kinds, signatures, determinism, and uniqueness. |
| `/docs/internals/providers/portable-and-target-operations/` | 15.6 | Portable core versus explicit target extension; discovery, compatibility, and source-level capability meaning. |
| `/docs/internals/providers/capability-examples/` | 15.7 | Clock/filesystem/HTTP/navigation/storage worked validation across value, lifecycle, target, and failure boundaries. |
| `/docs/internals/providers/existing-boundaries/` | 15.8 | Relationship to Effect services, standard modules, TypeScript interop, target adapters, manifests, locks, and runtime. |
| `/docs/internals/providers/schema-1-rejections/` | 15.9 | Complete logical contract schema-1 rejection checklist and diagnostic ownership. |
| `/docs/internals/providers/manifests/` | 15.10 | Provider identity/version/target/services/operations/runtime entries; deterministic form; unknown/missing/conflicting data. |
| `/docs/internals/providers/requirement-collection/` | 15.11 | Closed entry requirements, reachable operations, candidate visibility, dependencies, target and lock inputs. |
| `/docs/internals/providers/compatibility-filter/` | 15.12 | Contract, target, runtime, operation, type, and lifecycle filtering with reasons retained. |
| `/docs/internals/providers/deterministic-selection/` | 15.13 | Stable candidate ordering, explicit selection/lock precedence, ambiguity, and no environment-dependent guessing. |
| `/docs/internals/providers/preflight-diagnostics/` | 15.14 | Missing/incompatible/ambiguous provider diagnostics before execution, required metadata, and source/build locations. |
| `/docs/internals/providers/lock-and-build-metadata/` | 15.15 | Selected identity/version/digests, build artifact recording, stale detection, and reproducible examples. |
| `/docs/internals/providers/typescript-runtime-abi/` | 15.16 | ABI-v1 entrypoints, request/result ownership, operation table, service environment, and contract-to-backend separation. |
| `/docs/internals/providers/typescript-value-projection/` | 15.17 | Projection of every logical scalar/collection/record/opaque shape, validation, copies, identity, and errors. |
| `/docs/internals/providers/null-undefined-and-missing/` | 15.18 | Three distinct host states, legal carriers, optional presence, rejection, and no inferred Maybe/null collapsing. |
| `/docs/internals/providers/calls-and-result-envelopes/` | 15.19 | Call envelope identity/arguments/context; success/failure/defect/cancellation results; exactly-once completion; validation. |
| `/docs/internals/providers/opaque-handles/` | 15.20 | Handle creation/type/provider ownership, non-forgeability, lifetime, cross-operation use, and invalid/stale handle errors. |
| `/docs/internals/providers/conversion-responsibility/` | 15.21 | Compiler wrapper versus provider runtime conversion responsibilities and prohibition on double/unchecked conversion. |
| `/docs/internals/providers/cross-capability-projection/` | 15.22 | Passing logical values/handles across capabilities; identity/type compatibility; prohibited provider-specific coupling. |
| `/docs/internals/providers/abi-v1-rejections/` | 15.23 | Complete TypeScript ABI-v1 rejection checklist. |
| `/docs/internals/providers/lifecycle/` | 15.24 | Lifecycle state model, ownership, terminality, scope and host responsibilities. |
| `/docs/internals/providers/cold-effect-start/` | 15.25 | Cold Effect construction, exactly one provider start per execution, repeated execution, and no eager operation. |
| `/docs/internals/providers/terminal-outcomes/` | 15.26 | Success, typed failure, defect, cancellation classification and no channel substitution. |
| `/docs/internals/providers/cancellation-races/` | 15.27 | Notification protocol; before/during/after-start races; exactly-one terminal outcome; late completion disposal. |
| `/docs/internals/providers/resource-handoff/` | 15.28 | Acquire success, atomic scope registration, cancellation race, returned handle, and leak prevention. |
| `/docs/internals/providers/scope-cleanup/` | 15.29 | Cleanup trigger/order/idempotence; children/resources; graceful/forced shutdown; waiting and defects. |
| `/docs/internals/providers/causal-metadata/` | 15.30 | Initiation/host/observation metadata, operation/provider identity, safe source locations, and preserved causal groups. |
| `/docs/internals/providers/lifecycle-validation/` | 15.31 | Worked validation of operations with distinct one-shot/resource/cancellable lifecycles. |
| `/docs/internals/providers/lifecycle-schema-rejections/` | 15.32 | Complete lifecycle schema-1 rejection checklist. |
| `/docs/internals/providers/callback-stream-contract/` | 15.33 | Shared callback/stream model, registration state, consumer bridge, and separation from one-shot calls. |
| `/docs/internals/providers/one-shot-and-multi-shot/` | 15.34 | Cardinality, terminal events, repeated values, illegal calls, and adaptation choices. |
| `/docs/internals/providers/registration-and-removal/` | 15.35 | Registration completion, removal ownership/idempotence, race handling, and scope integration. |
| `/docs/internals/providers/demand-and-backpressure/` | 15.36 | Minimum demand protocol, producer permission, consumer accounting, bounded flow, and unsupported push behavior. |
| `/docs/internals/providers/overflow-and-protocol-violations/` | 15.37 | Overflow choices, explicit policy, typed/protocol failure classification, and forbidden silent loss. |
| `/docs/internals/providers/producer-consumer-cancellation/` | 15.38 | Both cancellation directions, pending demand, removal, terminal outcome, and late callback behavior. |
| `/docs/internals/providers/stream-signal-conversion/` | 15.39 | Provider callback to Stream/Signal responsibility; failure/lifetime/backpressure; no implicit semantic conversion. |
| `/docs/internals/providers/capability-boundaries/` | 15.40 | Isolation and composition of capabilities, handles, callbacks, provider state, and application-visible requirements. |
| `/docs/internals/providers/stream-schema-rejections/` | 15.41 | Complete callback/stream schema-1 rejection checklist. |
| `/docs/internals/providers/portable-surface/` | 15.42 | Stable portable surface, explicit target extension namespace, discovery and application portability implications. |
| `/docs/internals/providers/version-roles/` | 15.43 | Language, contract, schema, provider, adapter, runtime, target, and lock versions with independent compatibility roles. |
| `/docs/internals/providers/additive-and-breaking-changes/` | 15.44 | Exact classification of added operations/fields/types/semantics; required version changes and migration. |
| `/docs/internals/providers/handshake-diagnostics/` | 15.45 | Version/capability handshake, mismatch reporting, available ranges, provider/target identity, and pre-execution failure. |
| `/docs/internals/providers/conformance-cases/` | 15.46 | Case model, inputs/expected outcomes, lifecycle traces, portability, deterministic artifacts, and target matrices. |
| `/docs/internals/providers/backend-replacement/` | 15.47 | Contract-preserving backend substitution; what may differ; conformance evidence; no application API change. |
| `/docs/internals/providers/compatibility-schema-rejections/` | 15.48 | Complete compatibility schema-1 rejection checklist. |
| `/docs/internals/providers/eight-capability-validation/` | 15.49 | Final cross-capability validation matrix and required evidence. |
| `/docs/internals/providers/implementation-handoff/` | 15.50 | Dependency order into implementation epics without weakening contract ownership. |
| `/docs/internals/providers/design-audit/` | 15.51 | Recorded audit findings, resolved boundaries, remaining explicit decisions, and non-goals. |
| `/docs/internals/providers/application-capability-baseline/` | 15.52 | Rebased application capabilities, requirement identities, standard-module relationship, and migration. |
| `/docs/internals/providers/http-handler-bridge/` | 15.53 | Effectful server-handler adaptation, environment/error/body/resource/cancellation/shutdown flow, and provider/runtime ownership. |

## Database Package Reference

| Route | Source | Required semantic scope |
| --- | --- | --- |
| `/docs/packages/postgresql/identity-and-requirements/` | 16.1 | Package/service identity, Provider requirement, targets, configuration, and version/lock ownership. |
| `/docs/packages/postgresql/resources-and-values/` | 16.2 | Opaque pool/connection/prepared resources, immutable SQL values, acquisition, scope, cancellation, and release. |
| `/docs/packages/postgresql/query-and-decoding/` | 16.3 | Query/result/row model, parameter encoding, compositional row decoding, path/column errors, and stream/materialization behavior. |
| `/docs/packages/postgresql/transactions/` | 16.4 | Transaction resource, commit/rollback outcomes, nested behavior, failure/cancellation, and connection ownership. |
| `/docs/packages/postgresql/cursors/` | 16.5 | Cursor resource/Stream, demand/batching, ordering, cancellation, failure, and cleanup. |
| `/docs/packages/postgresql/failures-and-providers/` | 16.6 | Typed database/decode/config errors versus defects; Provider translation; target and portability boundaries. |
| `/docs/packages/sqlite/identity-and-requirements/` | 17.1 | Package/service identity, Provider requirement, target/file/memory configuration, version and lock ownership. |
| `/docs/packages/sqlite/database-and-parameters/` | 17.2 | Opaque database/statement resources, value/parameter model, preparation, binding, and lifetime. |
| `/docs/packages/sqlite/row-decoding/` | 17.3 | Result and row model, compositional decode, column/type/null errors, and ordering. |
| `/docs/packages/sqlite/transactions-and-resources/` | 17.4 | Transaction/savepoint behavior, commit/rollback, cancellation, resource nesting, and close. |
| `/docs/packages/sqlite/failures-targets-and-streams/` | 17.5 | Typed failures and provider mapping; target limitations; future Stream boundary without claiming unimplemented semantics. |

## Grammar Appendix

`/docs/language/grammar/` is a navigable grammar reference generated from Appendix A
without pretending that grammar alone defines semantics. Every production links
to its explanatory Language Reference leaf, and every leaf links back to the
relevant productions. The page also explains the appendix's syntactic
commitments, contextual tokens, and places where name resolution, typing, or
fixity resolution completes a parse that the grammar alone cannot decide.

## Full-reference coverage gate

The content architecture is complete only when every H2/H3 semantic unit in
`docs/spec/00-language.md` through `17-sqlite-package.md` has a stable route or
generated symbol child, every rule is assigned to an article section, and a
coverage check reports unmapped specification anchors as an error. The route
map is not permission to summarize: each leaf must satisfy the full article
contract before it can be published.
