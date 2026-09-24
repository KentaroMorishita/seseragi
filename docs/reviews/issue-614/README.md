# #614: failure reporting and correlated logs, first contract review

This is a Seseragi-only decision record against `origin/main` at
`6847a8d1` (2026-09-24). It does not authorize an Effect ABI change. The
explicit-correlation baseline below is executable; automatic spans remain a
separate decision.

## Existing ownership

| Concern | Existing owner | Consequence |
| --- | --- | --- |
| Recoverable failure | `Effect<R, E, A>` and an application-defined `E` | `mapError`, `recover`, and `attempt` see `E`; a display string is a projection, not a replacement for it. |
| Defect and cancellation | Effect runner and `RuntimeDiagnostic` | Neither becomes `E` or a successful result for logging convenience. Cancellation must remain observable even if a logger fails. |
| Foreign throw / rejection | `Js.Error` interop adapter | Its phase and causal stack already exist. Do not invent a second cause chain or copy its raw message into an application log automatically. |
| Application event | `std/log::LogEvent` and the required `Logger` service | The caller explicitly chooses message and fields. The logger owns timestamp, encoding, and destination, not domain-error wording or trace-ID generation. |
| Application-facing message | The application boundary | Locale, audience, recovery advice, and deliberate disclosure vary by application. No universal `Show<E>`-to-UI/message rule is sound. This does not replace the CLI's unhandled `RuntimeDiagnostic`. |

Sources: [Effect](../../spec/05-effects.md) §§5.6, 5.10–5.11;
[structured logging](../../spec/09-standard-library.md) §9.13;
[interop](../../spec/07-typescript-interop.md) §§7.3, 7.11, 7.15;
[`effect.ts`](../../../runtime/ts/src/effect.ts),
[`logger-service.ts`](../../../runtime/ts/src/logger-service.ts), and
[`service.ts`](../../../runtime/ts/src/service.ts).

An error code, if useful, belongs to a named domain error case or its explicit
reporting projection. It is stable within that application's public contract,
not an implicit identity synthesized by the compiler from a constructor name.
The reporting projection may yield an allowlisted code and safe scalar fields;
it must not serialize the entire `E`, `Js.Error`, stack, request payload, or
foreign cause. A separate pure projection chooses the user message. This does
not require a language-level `Error` trait: some `E` values are intentionally
internal, and one error may have several user messages by audience or locale.

## Correlation across execution boundaries

The runtime's `EffectContext` carries cancellation and resource-scope state,
not a public trace identity. Sequential composition retains the context;
`scoped` wraps it for a nested resource scope; `fork` creates a child execution
linked to parent cancellation. The `Logger` is an environment service shared
through `R`. None of these operations currently adds an operation ID or span
to `LogEvent`. A retained/asynchronous foreign callback has an explicit
hand-written resource adapter; its context cannot be inferred from the foreign
signature.

| Candidate | What it buys | Cost / risk | Decision for first slice |
| --- | --- | --- | --- |
| Explicit immutable correlation value carried by application code and copied into `LogEvent.fields` | Works today, including parallel children and adapter calls; IDs are chosen at an explicit host boundary | Repetition and no automatic nested spans | Use as the baseline to test whether a new standard API is needed. |
| Logger service decorator adding approved fields | Reduces repeated fields without changing `Effect` or `LogEvent` | The same environment may be shared by sibling Fibers; mutable decorator state would mix them | Consider only an immutable per-operation service instance, provided explicitly. |
| Fiber-local trace context and `withSpan` / `currentSpan` operations | Automatic propagation and nested spans | Changes Effect runner, scope, Fiber, Provider, and callback semantics; needs cancellation-safe cleanup and a cross-runtime contract | Defer until the baseline fails a concrete criterion. |
| Ambient global context or mandatory external telemetry SDK | Convenient for host integration | Hidden dependency, privacy risk, divergent browser/process semantics | Reject as the Seseragi language/standard-library contract. |

If automatic spans become justified, the invariant is: a root starts with no
identity unless supplied explicitly; a child Fiber inherits an immutable
parent context but receives its own span when an explicit span operation asks
for one; sibling updates never mutate each other; nested scope restores its
parent on success, failure, defect, and cancellation; a host callback crosses
the boundary only with an explicit captured context. A provider may attach
external transport metadata only through its own typed adapter. No user data
or foreign error text is attached by default.

## Executable proof, before adding an API

The self-contained [`log-correlation`](../../../examples/spec/fixtures/projects/log-correlation)
project now:

1. Defines a small domain-error ADT and two **pure** projections: one
   user-facing message, one allowlisted report code/fields. Keep the original
   ADT until the application boundary. The ADT deliberately carries no raw
   input; validation turns a private input into a safe error case.
2. Forks two operations with different explicit correlation IDs. Each logs
   start and outcome using the current `std/log` service; one succeeds and one
   yields a typed failure, which the caller maps to a safe user message.
3. Asserts the CLI's stdout and structured stderr exactly. A test also rejects
   the `private-` sentinel from either output. The two interleaved operations
   keep their own IDs without mutable Logger state.

The fixture uses no foreign adapter, so it does not claim to prove redaction
of a raw foreign error or callback propagation. Cancellation correlation and
Provider propagation are not established by this test. Those cases should be
tested before introducing an automatic span API, not assumed to work from the
fixture's success.

The existing [`console-logger` probe](../../../runtime/ts/probes/console-logger.ts)
also passes: it proves logging is cold, routes through Logger rather than
Console, preserves field order, and keeps logger failure separate from a raw
defect. It does not prove correlation by itself.

## Narrow conclusion

Do not add a universal Error type, an HKT/trait abstraction for user messages,
or automatic trace state now. The fixture and the §9.13 spec paragraph fix the
explicit-correlation and safe-projection baseline. A runtime span API is a
separate decision contingent on a concrete Provider/callback or cancellation
case showing loss of semantics or unacceptable duplication.
