import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { unfold as _ssrg_iterator_unfold, collectMap as _ssrg_iterator_comprehend, type Iterator as Iterator } from "@seseragi/runtime/iterator"
import { reduce as _ssrg_range_reduce, inclusive as _ssrg_range_inclusive } from "@seseragi/runtime/range"
import { intOrd as _ssrg_int_ord_dictionary, intEq as _ssrg_int_eq_dictionary } from "@seseragi/runtime/equality"
import { Just as _ssrg_maybe_Just, Nothing as _ssrg_maybe_Nothing } from "@seseragi/runtime/sum"
import { add as _ssrg_int_add, multiply as _ssrg_int_multiply, remainder as _ssrg_int_remainder } from "@seseragi/runtime/int"
import { reduce as _ssrg_array_reduce } from "@seseragi/runtime/array"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source132 = { file: "main.ssrg", line: 6, column: 5, uri: "seseragi://artifact/user-iterable-comprehension", start: 132, end: 159 };
const $ssrg$source147 = { file: "main.ssrg", line: 6, column: 20, uri: "seseragi://artifact/user-iterable-comprehension", start: 147, end: 158 };
const $ssrg$source310 = { file: "main.ssrg", line: 13, column: 26, uri: "seseragi://artifact/user-iterable-comprehension", start: 310, end: 334 };
const $ssrg$source318 = { file: "main.ssrg", line: 13, column: 34, uri: "seseragi://artifact/user-iterable-comprehension", start: 318, end: 331 };
const $ssrg$source508 = { file: "main.ssrg", line: 21, column: 26, uri: "seseragi://artifact/user-iterable-comprehension", start: 508, end: 539 };
const $ssrg$source529 = { file: "main.ssrg", line: 21, column: 47, uri: "seseragi://artifact/user-iterable-comprehension", start: 529, end: 538 };
const $ssrg$source600 = { file: "main.ssrg", line: 26, column: 4, uri: "seseragi://artifact/user-iterable-comprehension", start: 600, end: 613 };
const $ssrg$source633 = { file: "main.ssrg", line: 26, column: 37, uri: "seseragi://artifact/user-iterable-comprehension", start: 633, end: 642 };
const $ssrg$source695 = { file: "main.ssrg", line: 29, column: 3, uri: "seseragi://artifact/user-iterable-comprehension", start: 695, end: 707 };
const $ssrg$source766 = { file: "main.ssrg", line: 33, column: 3, uri: "seseragi://artifact/user-iterable-comprehension", start: 766, end: 793 };
const $ssrg$source847 = { file: "main.ssrg", line: 38, column: 5, uri: "seseragi://artifact/user-iterable-comprehension", start: 847, end: 867 };
const $ssrg$source889 = { file: "main.ssrg", line: 39, column: 5, uri: "seseragi://artifact/user-iterable-comprehension", start: 889, end: 904 };
const $ssrg$source1097 = { file: "main.ssrg", line: 49, column: 3, uri: "seseragi://artifact/user-iterable-comprehension", start: 1097, end: 1108 };
$ssrg$assertUnicodeVersion("17.0.0")

export type Countdown =
  | { readonly tag: "Countdown"; readonly value: number };
export const Countdown = (value: number): Countdown => ({ tag: "Countdown", value } as const);
export const __ssrg$instance$Iterable$0 = { "iterate": (values: Countdown) => (($ssrg_match: Countdown): Iterator<number> => $ssrg_match.tag === "Countdown" ? ((limit: number): Iterator<number> => $ssrg$atSource($ssrg$source310, () => _ssrg_iterator_unfold($ssrg$atSource($ssrg$source318, () => advance(limit), false), 1), false))($ssrg_match.value) : ((): never => { throw new globalThis.Error("non-exhaustive Seseragi match"); })())(values) } as const;
export const __ssrg$instance$Reducible$1 = (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => ({ ...__ssrg$evidence$0, "reduce": <B,>(initial: B) => (step: (argument: B) => (argument: number) => B) => (values: Countdown) => (($ssrg_match: Countdown): B => $ssrg_match.tag === "Countdown" ? ((limit: number): B => $ssrg$atSource($ssrg$source508, () => _ssrg_range_reduce(initial, step, $ssrg$atSource($ssrg$source529, () => _ssrg_range_inclusive(1, limit), false)), false))($ssrg_match.value) : ((): never => { throw new globalThis.Error("non-exhaustive Seseragi match"); })())(values) }) as const;
const advance = (limit: number) => (current: number) => (_ssrg_int_ord_dictionary["compare"](current)(limit))["tag"] !== "Greater" ? $ssrg$atSource($ssrg$source132, () => _ssrg_maybe_Just([current, _ssrg_int_add(current, 1, $ssrg$source147)] as const), false) : _ssrg_maybe_Nothing
const oddSquares = (values: Countdown) => _ssrg_iterator_comprehend(__ssrg$instance$Iterable$0["iterate"](values), (value) => _ssrg_int_eq_dictionary["eq"](_ssrg_int_remainder(value, 2, $ssrg$source633))(0) === false, (value) => _ssrg_int_multiply(value, value, $ssrg$source600))
const addInt = (left: number) => (right: number) => _ssrg_int_add(left, right, $ssrg$source695)
const total = <C,>(values: C) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => $ssrg$atSource($ssrg$source766, () => __ssrg$evidence$0["reduce"](0)(addInt)(values), false)
const totals = (values: Countdown) => [$ssrg$atSource($ssrg$source847, () => _ssrg_array_reduce(0, (((_argument0) => (_argument1) => _ssrg_int_add(_argument0, _argument1)) as (argument: number) => (argument: number) => number), $ssrg$atSource($ssrg$source847, () => oddSquares(values), false)), false), $ssrg$atSource($ssrg$source889, () => ((total<Countdown>(values)(__ssrg$instance$Reducible$1(__ssrg$instance$Iterable$0))) as number), false)] as const
const renderTotals = (values: readonly [number, number]) => (($ssrg_match: readonly [number, number]): string => $ssrg_match[0] === 35 && $ssrg_match[1] === 15 ? "user collection totals: 35 / 15" : "unexpected user collection totals")(values)
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source1097, () => _ssrg_console_println($ssrg$atSource($ssrg$source1097, () => renderTotals($ssrg$atSource($ssrg$source1097, () => totals($ssrg$atSource($ssrg$source1097, () => Countdown(5), false)), false)), false)), true)
