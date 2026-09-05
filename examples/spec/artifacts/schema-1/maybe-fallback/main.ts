import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { Nothing as _ssrg_maybe_Nothing, Just as _ssrg_maybe_Just } from "@seseragi/runtime/sum"
import { divide as _ssrg_int_divide } from "@seseragi/runtime/int"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
$ssrg$assertUnicodeVersion("17.0.0")

const resolve = <A,>(value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: A }) => (fallback: A) => (($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: A }): A => $ssrg_match.tag === "Just" ? (($ssrg$fallbackValue: A): A => $ssrg$fallbackValue)($ssrg_match.value) : fallback)(value)
const fail = (unit: undefined) => _ssrg_int_divide(1, 0)
export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_console_println((($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: string }): string => $ssrg_match.tag === "Just" ? (($ssrg$fallbackValue: string): string => $ssrg$fallbackValue)($ssrg_match.value) : (($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: string }): string => $ssrg_match.tag === "Just" ? (($ssrg$fallbackValue: string): string => $ssrg$fallbackValue)($ssrg_match.value) : "anonymous")(requested))(cached)), () => _ssrg_effect_flatMap(_ssrg_console_println(((_ssrg_show_intShow["show"]((($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }): number => $ssrg_match.tag === "Just" ? (($ssrg$fallbackValue: number): number => $ssrg$fallbackValue)($ssrg_match.value) : fail(undefined))(_ssrg_maybe_Just(7)))) as string)), () => _ssrg_effect_flatMap(_ssrg_console_println(((_ssrg_show_intShow["show"]((($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }): number => $ssrg_match.tag === "Just" ? (($ssrg$fallbackValue: number): number => $ssrg$fallbackValue)($ssrg_match.value) : 8)(_ssrg_maybe_Nothing))) as string)), () => _ssrg_console_println(((_ssrg_show_intShow["show"](resolve(_ssrg_maybe_Just(9))(0))) as string)))))
const cached: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: string } = _ssrg_maybe_Nothing;
const requested: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: string } = _ssrg_maybe_Just("request");
