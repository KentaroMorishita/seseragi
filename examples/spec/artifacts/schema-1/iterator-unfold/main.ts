import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { intOrd as _ssrg_int_ord_dictionary, intEq as _ssrg_int_eq_dictionary } from "@seseragi/runtime/equality"
import { Just as _ssrg_maybe_Just, Nothing as _ssrg_maybe_Nothing } from "@seseragi/runtime/sum"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
import { next as _ssrg_iterator_next, unfold as _ssrg_iterator_unfold, type Iterator as Iterator } from "@seseragi/runtime/iterator"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source71 = { file: "main.ssrg", line: 3, column: 5, uri: "seseragi://artifact/iterator-unfold", start: 71, end: 98 };
const $ssrg$source86 = { file: "main.ssrg", line: 3, column: 20, uri: "seseragi://artifact/iterator-unfold", start: 86, end: 97 };
const $ssrg$source174 = { file: "main.ssrg", line: 8, column: 10, uri: "seseragi://artifact/iterator-unfold", start: 174, end: 187 };
const $ssrg$source189 = { file: "main.ssrg", line: 8, column: 25, uri: "seseragi://artifact/iterator-unfold", start: 189, end: 202 };
const $ssrg$source271 = { file: "main.ssrg", line: 10, column: 10, uri: "seseragi://artifact/iterator-unfold", start: 271, end: 284 };
const $ssrg$source305 = { file: "main.ssrg", line: 11, column: 16, uri: "seseragi://artifact/iterator-unfold", start: 305, end: 318 };
const $ssrg$source320 = { file: "main.ssrg", line: 11, column: 31, uri: "seseragi://artifact/iterator-unfold", start: 320, end: 334 };
const $ssrg$source408 = { file: "main.ssrg", line: 13, column: 16, uri: "seseragi://artifact/iterator-unfold", start: 408, end: 429 };
const $ssrg$source678 = { file: "main.ssrg", line: 25, column: 3, uri: "seseragi://artifact/iterator-unfold", start: 678, end: 691 };
$ssrg$assertUnicodeVersion("17.0.0")

const step = (current: number) => (_ssrg_int_ord_dictionary["compare"](current)(3))["tag"] !== "Greater" ? $ssrg$atSource($ssrg$source71, () => _ssrg_maybe_Just([current, _ssrg_int_add(current, 1, $ssrg$source86)] as const), false) : _ssrg_maybe_Nothing
const render = (iterator: Iterator<number>) => (($ssrg_match: readonly [{ readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: readonly [number, Iterator<number>] }, { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: readonly [number, Iterator<number>] }]): string => $ssrg_match[0].tag === "Just" && $ssrg_match[1].tag === "Just" ? ((left: number, leftRest: Iterator<number>, right: number, rightRest: Iterator<number>): string => $ssrg$atSource($ssrg$source271, () => _ssrg_int_eq_dictionary["eq"](left)(right), false) ? (($ssrg_match: readonly [{ readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: readonly [number, Iterator<number>] }, { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: readonly [number, Iterator<number>] }]): string => $ssrg_match[0].tag === "Just" && $ssrg_match[1].tag === "Just" ? ((leftNext: number, rightNext: number): string => $ssrg$atSource($ssrg$source408, () => _ssrg_int_eq_dictionary["eq"](leftNext)(rightNext), false) ? "persistent iterator: 1 then 2" : "iterator rest changed")($ssrg_match[0].value[0], $ssrg_match[1].value[0]) : "iterator ended early")([$ssrg$atSource($ssrg$source305, () => _ssrg_iterator_next(leftRest), false), $ssrg$atSource($ssrg$source320, () => _ssrg_iterator_next(rightRest), false)] as const) : "iterator changed")($ssrg_match[0].value[0], $ssrg_match[0].value[1], $ssrg_match[1].value[0], $ssrg_match[1].value[1]) : "iterator changed")([$ssrg$atSource($ssrg$source174, () => _ssrg_iterator_next(iterator), false), $ssrg$atSource($ssrg$source189, () => _ssrg_iterator_next(iterator), false)] as const)
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source678, () => _ssrg_console_println($ssrg$atSource($ssrg$source678, () => render($ssrg$atSource($ssrg$source678, () => _ssrg_iterator_unfold(step, 1), false)), false)), true)
