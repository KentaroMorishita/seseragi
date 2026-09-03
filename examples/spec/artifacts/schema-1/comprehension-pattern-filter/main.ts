import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { collectMap as _ssrg_array_comprehend, reduce as _ssrg_array_reduce } from "@seseragi/runtime/array"
import { Just as _ssrg_maybe_Just, Nothing as _ssrg_maybe_Nothing } from "@seseragi/runtime/sum"
import { intEq as _ssrg_int_eq_dictionary } from "@seseragi/runtime/equality"
import { remainder as _ssrg_int_remainder, add as _ssrg_int_add } from "@seseragi/runtime/int"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
$ssrg$assertUnicodeVersion("17.0.0")

const presentValues = (unit: undefined) => _ssrg_array_comprehend([_ssrg_maybe_Just(1), _ssrg_maybe_Nothing, _ssrg_maybe_Just(3), _ssrg_maybe_Just(4)], ($ssrg_item0) => (($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }): boolean => $ssrg_match.tag === "Just" && ((value: number): boolean => _ssrg_int_eq_dictionary["eq"](_ssrg_int_remainder(value, 2))(1))($ssrg_match.value) ? ((value: number): boolean => true)($ssrg_match.value) : false)($ssrg_item0), ($ssrg_item0) => (($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }): number => $ssrg_match.tag === "Just" ? ((value: number): number => value)($ssrg_match.value) : ((): never => { throw new Error("non-exhaustive Seseragi match"); })())($ssrg_item0))
const matchingValues = (unit: undefined) => _ssrg_array_comprehend([[1, 10] as const, [2, 20] as const, [1, 30] as const], ($ssrg_item0) => (($ssrg_match: readonly [number, number]): boolean => $ssrg_match[0] === 1 ? ((value: number): boolean => true)($ssrg_match[1]) : false)($ssrg_item0), ($ssrg_item0) => (($ssrg_match: readonly [number, number]): number => $ssrg_match[0] === 1 ? ((value: number): number => value)($ssrg_match[1]) : ((): never => { throw new Error("non-exhaustive Seseragi match"); })())($ssrg_item0))
const presentTotal = (unit: undefined) => _ssrg_array_reduce(0, (_argument0) => (_argument1) => _ssrg_int_add(_argument0, _argument1), presentValues(undefined))
const matchingTotal = (unit: undefined) => _ssrg_array_reduce(0, (_argument0) => (_argument1) => _ssrg_int_add(_argument0, _argument1), matchingValues(undefined))
const renderTotals = (totals: readonly [number, number]) => (($ssrg_match: readonly [number, number]): string => $ssrg_match[0] === 4 && $ssrg_match[1] === 40 ? "pattern-filter totals: 4 / 40" : "unexpected pattern-filter totals")(totals)
export const main = (_unit: undefined) => _ssrg_console_println(renderTotals([presentTotal(undefined), matchingTotal(undefined)] as const))
