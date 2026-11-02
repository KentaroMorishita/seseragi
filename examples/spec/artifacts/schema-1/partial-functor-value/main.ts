import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { Nothing as _ssrg_maybe_Nothing, Just as _ssrg_maybe_Just } from "@seseragi/runtime/sum"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source227 = { file: "main.ssrg", line: 9, column: 20, uri: "seseragi://artifact/partial-functor-value", start: 227, end: 240 };
const $ssrg$source234 = { file: "main.ssrg", line: 9, column: 27, uri: "seseragi://artifact/partial-functor-value", start: 234, end: 240 };
const $ssrg$source283 = { file: "main.ssrg", line: 13, column: 34, uri: "seseragi://artifact/partial-functor-value", start: 283, end: 292 };
const $ssrg$source389 = { file: "main.ssrg", line: 19, column: 3, uri: "seseragi://artifact/partial-functor-value", start: 389, end: 401 };
const $ssrg$source572 = { file: "main.ssrg", line: 29, column: 3, uri: "seseragi://artifact/partial-functor-value", start: 572, end: 579 };
const $ssrg$source598 = { file: "main.ssrg", line: 30, column: 19, uri: "seseragi://artifact/partial-functor-value", start: 598, end: 611 };
$ssrg$assertUnicodeVersion("17.0.0")

export const __ssrg$instance$Functor$0 = { "map": <A, B,>(f: (argument: A) => B) => (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: A }) => (($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: A }): { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: B } => $ssrg_match.tag === "Nothing" ? _ssrg_maybe_Nothing : $ssrg_match.tag === "Just" ? ((item: A): { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: B } => $ssrg$atSource($ssrg$source227, () => _ssrg_maybe_Just($ssrg$atSource($ssrg$source234, () => f(item), false)), false))($ssrg_match.value) : ((): never => { throw new globalThis.Error("non-exhaustive Seseragi match"); })())(value) } as const;
const increment = (value: number) => _ssrg_int_add(value, 1, $ssrg$source283)
const applyMapper = (mapper: (argument: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }) => { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }) => (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }) => $ssrg$atSource($ssrg$source389, () => mapper(value), false)
const render = (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }) => (($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }): string => $ssrg_match.tag === "Nothing" ? "Nothing" : $ssrg_match.tag === "Just" && $ssrg_match.value === 42 ? "Just 42" : "Just another value")(value)
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source572, () => _ssrg_console_println($ssrg$atSource($ssrg$source572, () => render($ssrg$atSource($ssrg$source572, () => applyMapper($ssrg$atSource($ssrg$source598, () => ((__ssrg$instance$Functor$0["map"](increment)) as (argument: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }) => { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }), false))($ssrg$atSource($ssrg$source572, () => _ssrg_maybe_Just(41), false)), false)), false)), true)
