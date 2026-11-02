import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { Nothing as _ssrg_maybe_Nothing, Just as _ssrg_maybe_Just } from "@seseragi/runtime/sum"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source227 = { file: "main.ssrg", line: 9, column: 20, uri: "seseragi://artifact/functor-maybe", start: 227, end: 240 };
const $ssrg$source234 = { file: "main.ssrg", line: 9, column: 27, uri: "seseragi://artifact/functor-maybe", start: 234, end: 240 };
const $ssrg$source333 = { file: "main.ssrg", line: 16, column: 3, uri: "seseragi://artifact/functor-maybe", start: 333, end: 349 };
const $ssrg$source384 = { file: "main.ssrg", line: 19, column: 34, uri: "seseragi://artifact/functor-maybe", start: 384, end: 393 };
const $ssrg$source573 = { file: "main.ssrg", line: 30, column: 5, uri: "seseragi://artifact/functor-maybe", start: 573, end: 580 };
$ssrg$assertUnicodeVersion("17.0.0")

interface $ssrg$Kind<Args extends readonly unknown[] = readonly unknown[]> { readonly args: Args; readonly type: unknown }
type $ssrg$Invoke<F extends $ssrg$Kind, Args extends readonly unknown[]> = (F & { readonly args: Args })["type"];
type $ssrg$Apply<F extends $ssrg$Kind, Args extends F["args"]> = $ssrg$Invoke<F, Args>;
export const __ssrg$instance$Functor$0 = { "map": <A, B,>(f: (argument: A) => B) => (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: A }) => (($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: A }): { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: B } => $ssrg_match.tag === "Nothing" ? _ssrg_maybe_Nothing : $ssrg_match.tag === "Just" ? ((item: A): { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: B } => $ssrg$atSource($ssrg$source227, () => _ssrg_maybe_Just($ssrg$atSource($ssrg$source234, () => f(item), false)), false))($ssrg_match.value) : ((): never => { throw new globalThis.Error("non-exhaustive Seseragi match"); })())(value) } as const;
const transform = <F extends { readonly args: readonly [unknown]; readonly type: unknown }, A, B,>(f: (argument: A) => B) => (value: $ssrg$Apply<F, readonly [A]>) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => $ssrg$atSource($ssrg$source333, () => ((__ssrg$evidence$0["map"](f)(value)) as $ssrg$Apply<F, readonly [B]>), false)
const increment = (value: number) => _ssrg_int_add(value, 1, $ssrg$source384)
const render = (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }) => (($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }): string => $ssrg_match.tag === "Nothing" ? "Nothing" : $ssrg_match.tag === "Just" && $ssrg_match.value === 42 ? "Just 42" : "Just another value")(value)
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source573, () => _ssrg_console_println($ssrg$atSource($ssrg$source573, () => render($ssrg$atSource($ssrg$source573, () => ((transform(increment)($ssrg$atSource($ssrg$source573, () => _ssrg_maybe_Just(41), false))(__ssrg$instance$Functor$0)) as { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }), false)), false)), true)
