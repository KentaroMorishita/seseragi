import { Countdown, __ssrg$instance$Iterable$0, __ssrg$instance$Reducible$1, type Countdown as Countdown_1 } from "./domain.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { multiply as _ssrg_int_multiply, remainder as _ssrg_int_remainder, add as _ssrg_int_add } from "@seseragi/runtime/int"
import { collectMap as _ssrg_iterator_comprehend } from "@seseragi/runtime/iterator"
import { intEq as _ssrg_int_eq_dictionary } from "@seseragi/runtime/equality"
import { reduce as _ssrg_array_reduce } from "@seseragi/runtime/array"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source89 = { file: "project/imported-iterable-comprehension/src/main.ssrg", line: 4, column: 4, uri: "seseragi://fixture/imported-iterable-comprehension::main", start: 89, end: 102 };
const $ssrg$source122 = { file: "project/imported-iterable-comprehension/src/main.ssrg", line: 4, column: 37, uri: "seseragi://fixture/imported-iterable-comprehension::main", start: 122, end: 131 };
const $ssrg$source184 = { file: "project/imported-iterable-comprehension/src/main.ssrg", line: 7, column: 3, uri: "seseragi://fixture/imported-iterable-comprehension::main", start: 184, end: 196 };
const $ssrg$source255 = { file: "project/imported-iterable-comprehension/src/main.ssrg", line: 11, column: 3, uri: "seseragi://fixture/imported-iterable-comprehension::main", start: 255, end: 282 };
const $ssrg$source336 = { file: "project/imported-iterable-comprehension/src/main.ssrg", line: 16, column: 5, uri: "seseragi://fixture/imported-iterable-comprehension::main", start: 336, end: 356 };
const $ssrg$source378 = { file: "project/imported-iterable-comprehension/src/main.ssrg", line: 17, column: 5, uri: "seseragi://fixture/imported-iterable-comprehension::main", start: 378, end: 393 };
const $ssrg$source594 = { file: "project/imported-iterable-comprehension/src/main.ssrg", line: 27, column: 3, uri: "seseragi://fixture/imported-iterable-comprehension::main", start: 594, end: 605 };
$ssrg$assertUnicodeVersion("17.0.0")

const oddSquares = (values: Countdown) => _ssrg_iterator_comprehend(__ssrg$instance$Iterable$0["iterate"](values), (value) => _ssrg_int_eq_dictionary["eq"](_ssrg_int_remainder(value, 2, $ssrg$source122))(0) === false, (value) => _ssrg_int_multiply(value, value, $ssrg$source89))
const addInt = (left: number) => (right: number) => _ssrg_int_add(left, right, $ssrg$source184)
const total = <C,>(values: C) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => $ssrg$atSource($ssrg$source255, () => __ssrg$evidence$0["reduce"](0)(addInt)(values), false)
const totals = (values: Countdown) => [$ssrg$atSource($ssrg$source336, () => _ssrg_array_reduce(0, (((_argument0) => (_argument1) => _ssrg_int_add(_argument0, _argument1)) as (argument: number) => (argument: number) => number), $ssrg$atSource($ssrg$source336, () => oddSquares(values), false)), false), $ssrg$atSource($ssrg$source378, () => ((total<Countdown>(values)(__ssrg$instance$Reducible$1(__ssrg$instance$Iterable$0))) as number), false)] as const
const renderTotals = (values: readonly [number, number]) => (($ssrg_match: readonly [number, number]): string => $ssrg_match[0] === 35 && $ssrg_match[1] === 15 ? "imported collection totals: 35 / 15" : "unexpected imported collection totals")(values)
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source594, () => _ssrg_console_println($ssrg$atSource($ssrg$source594, () => renderTotals($ssrg$atSource($ssrg$source594, () => totals($ssrg$atSource($ssrg$source594, () => Countdown(5), false)), false)), false)), true)
