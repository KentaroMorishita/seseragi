import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { Nothing as _ssrg_maybe_Nothing, Just as _ssrg_maybe_Just } from "@seseragi/runtime/sum"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
const $ssrg$source227 = { file: "project/imported-functor-dispatch/src/domain.ssrg", line: 9, column: 20, uri: "seseragi://fixture/imported-functor-dispatch::domain", start: 227, end: 240 };
const $ssrg$source234 = { file: "project/imported-functor-dispatch/src/domain.ssrg", line: 9, column: 27, uri: "seseragi://fixture/imported-functor-dispatch::domain", start: 234, end: 240 };
const $ssrg$source283 = { file: "project/imported-functor-dispatch/src/domain.ssrg", line: 13, column: 34, uri: "seseragi://fixture/imported-functor-dispatch::domain", start: 283, end: 292 };
const $ssrg$source369 = { file: "project/imported-functor-dispatch/src/domain.ssrg", line: 19, column: 3, uri: "seseragi://fixture/imported-functor-dispatch::domain", start: 369, end: 388 };
$ssrg$assertUnicodeVersion("17.0.0")

interface $ssrg$Kind<Args extends readonly unknown[] = readonly unknown[]> { readonly args: Args; readonly type: unknown }
type $ssrg$Invoke<F extends $ssrg$Kind, Args extends readonly unknown[]> = (F & { readonly args: Args })["type"];
type $ssrg$Apply<F extends $ssrg$Kind, Args extends F["args"]> = $ssrg$Invoke<F, Args>;
export const __ssrg$instance$Functor$0 = { "map": <A, B,>(f: (argument: A) => B) => (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: A }) => (($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: A }): { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: B } => $ssrg_match.tag === "Nothing" ? _ssrg_maybe_Nothing : $ssrg_match.tag === "Just" ? ((item: A): { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: B } => $ssrg$atSource($ssrg$source227, () => _ssrg_maybe_Just($ssrg$atSource($ssrg$source234, () => f(item), false)), false))($ssrg_match.value) : ((): never => { throw new globalThis.Error("non-exhaustive Seseragi match"); })())(value) } as const;
const increment = (value: number) => _ssrg_int_add(value, 1, $ssrg$source283)
export const incrementAll = <F extends { readonly args: readonly [unknown]; readonly type: unknown },>(value: $ssrg$Apply<F, readonly [number]>) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => $ssrg$atSource($ssrg$source369, () => ((__ssrg$evidence$0["map"](increment)(value)) as $ssrg$Apply<F, readonly [number]>), false)
