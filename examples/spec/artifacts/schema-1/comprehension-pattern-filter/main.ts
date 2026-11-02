import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { collectMap as _ssrg_array_comprehend, reduce as _ssrg_array_reduce } from "@seseragi/runtime/array"
import { Just as _ssrg_maybe_Just, Nothing as _ssrg_maybe_Nothing } from "@seseragi/runtime/sum"
import { intEq as _ssrg_int_eq_dictionary } from "@seseragi/runtime/equality"
import { remainder as _ssrg_int_remainder, add as _ssrg_int_add } from "@seseragi/runtime/int"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source74 = { file: "main.ssrg", line: 3, column: 22, uri: "seseragi://artifact/comprehension-pattern-filter", start: 74, end: 80 };
const $ssrg$source91 = { file: "main.ssrg", line: 3, column: 39, uri: "seseragi://artifact/comprehension-pattern-filter", start: 91, end: 97 };
const $ssrg$source99 = { file: "main.ssrg", line: 3, column: 47, uri: "seseragi://artifact/comprehension-pattern-filter", start: 99, end: 105 };
const $ssrg$source114 = { file: "main.ssrg", line: 4, column: 7, uri: "seseragi://artifact/comprehension-pattern-filter", start: 114, end: 123 };
const $ssrg$source273 = { file: "main.ssrg", line: 11, column: 3, uri: "seseragi://artifact/comprehension-pattern-filter", start: 273, end: 289 };
const $ssrg$source348 = { file: "main.ssrg", line: 15, column: 3, uri: "seseragi://artifact/comprehension-pattern-filter", start: 348, end: 365 };
const $ssrg$source568 = { file: "main.ssrg", line: 25, column: 3, uri: "seseragi://artifact/comprehension-pattern-filter", start: 568, end: 621 };
const $ssrg$source569 = { file: "main.ssrg", line: 25, column: 4, uri: "seseragi://artifact/comprehension-pattern-filter", start: 569, end: 584 };
const $ssrg$source586 = { file: "main.ssrg", line: 25, column: 21, uri: "seseragi://artifact/comprehension-pattern-filter", start: 586, end: 602 };
$ssrg$assertUnicodeVersion("17.0.0")

const presentValues = (unit: undefined) => _ssrg_array_comprehend([$ssrg$atSource($ssrg$source74, () => _ssrg_maybe_Just(1), false), _ssrg_maybe_Nothing, $ssrg$atSource($ssrg$source91, () => _ssrg_maybe_Just(3), false), $ssrg$atSource($ssrg$source99, () => _ssrg_maybe_Just(4), false)], ($ssrg_item0) => (($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }): boolean => $ssrg_match.tag === "Just" && ((value: number): boolean => $ssrg$atSource($ssrg$source114, () => _ssrg_int_eq_dictionary["eq"](_ssrg_int_remainder(value, 2, $ssrg$source114))(1), false))($ssrg_match.value) ? ((value: number): boolean => true)($ssrg_match.value) : false)($ssrg_item0), ($ssrg_item0) => (($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }): number => $ssrg_match.tag === "Just" ? ((value: number): number => value)($ssrg_match.value) : ((): never => { throw new globalThis.Error("non-exhaustive Seseragi match"); })())($ssrg_item0))
const matchingValues = (unit: undefined) => _ssrg_array_comprehend([[1, 10] as const, [2, 20] as const, [1, 30] as const], ($ssrg_item0) => (($ssrg_match: readonly [number, number]): boolean => $ssrg_match[0] === 1 ? ((value: number): boolean => true)($ssrg_match[1]) : false)($ssrg_item0), ($ssrg_item0) => (($ssrg_match: readonly [number, number]): number => $ssrg_match[0] === 1 ? ((value: number): number => value)($ssrg_match[1]) : ((): never => { throw new globalThis.Error("non-exhaustive Seseragi match"); })())($ssrg_item0))
const presentTotal = (unit: undefined) => $ssrg$atSource($ssrg$source273, () => _ssrg_array_reduce(0, (((_argument0) => (_argument1) => _ssrg_int_add(_argument0, _argument1)) as (argument: number) => (argument: number) => number), $ssrg$atSource($ssrg$source273, () => presentValues(undefined), false)), false)
const matchingTotal = (unit: undefined) => $ssrg$atSource($ssrg$source348, () => _ssrg_array_reduce(0, (((_argument0) => (_argument1) => _ssrg_int_add(_argument0, _argument1)) as (argument: number) => (argument: number) => number), $ssrg$atSource($ssrg$source348, () => matchingValues(undefined), false)), false)
const renderTotals = (totals: readonly [number, number]) => (($ssrg_match: readonly [number, number]): string => $ssrg_match[0] === 4 && $ssrg_match[1] === 40 ? "pattern-filter totals: 4 / 40" : "unexpected pattern-filter totals")(totals)
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source568, () => _ssrg_console_println($ssrg$atSource($ssrg$source568, () => renderTotals([$ssrg$atSource($ssrg$source569, () => presentTotal(undefined), false), $ssrg$atSource($ssrg$source586, () => matchingTotal(undefined), false)] as const), false)), true)
