import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { Nothing as _ssrg_maybe_Nothing, Just as _ssrg_maybe_Just } from "@seseragi/runtime/sum"
import { divide as _ssrg_int_divide } from "@seseragi/runtime/int"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
const $ssrg$source98 = { file: "main.ssrg", line: 2, column: 29, uri: "seseragi://artifact/maybe-fallback", start: 98, end: 103 };
const $ssrg$source171 = { file: "main.ssrg", line: 4, column: 32, uri: "seseragi://artifact/maybe-fallback", start: 171, end: 185 };
const $ssrg$source214 = { file: "main.ssrg", line: 6, column: 3, uri: "seseragi://artifact/maybe-fallback", start: 214, end: 258 };
const $ssrg$source261 = { file: "main.ssrg", line: 7, column: 3, uri: "seseragi://artifact/maybe-fallback", start: 261, end: 295 };
const $ssrg$source271 = { file: "main.ssrg", line: 7, column: 13, uri: "seseragi://artifact/maybe-fallback", start: 271, end: 295 };
const $ssrg$source277 = { file: "main.ssrg", line: 7, column: 19, uri: "seseragi://artifact/maybe-fallback", start: 277, end: 283 };
const $ssrg$source287 = { file: "main.ssrg", line: 7, column: 29, uri: "seseragi://artifact/maybe-fallback", start: 287, end: 294 };
const $ssrg$source298 = { file: "main.ssrg", line: 8, column: 3, uri: "seseragi://artifact/maybe-fallback", start: 298, end: 327 };
const $ssrg$source308 = { file: "main.ssrg", line: 8, column: 13, uri: "seseragi://artifact/maybe-fallback", start: 308, end: 327 };
const $ssrg$source330 = { file: "main.ssrg", line: 9, column: 3, uri: "seseragi://artifact/maybe-fallback", start: 330, end: 365 };
const $ssrg$source340 = { file: "main.ssrg", line: 9, column: 13, uri: "seseragi://artifact/maybe-fallback", start: 340, end: 365 };
const $ssrg$source346 = { file: "main.ssrg", line: 9, column: 19, uri: "seseragi://artifact/maybe-fallback", start: 346, end: 364 };
const $ssrg$source355 = { file: "main.ssrg", line: 9, column: 28, uri: "seseragi://artifact/maybe-fallback", start: 355, end: 361 };
$ssrg$assertUnicodeVersion("17.0.0")

const resolve = <A,>(value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: A }) => (fallback: A) => (($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: A }): A => $ssrg_match.tag === "Just" ? (($ssrg$fallbackValue: A): A => $ssrg$fallbackValue)($ssrg_match.value) : fallback)(value)
const fail = (unit: undefined) => _ssrg_int_divide(1, 0, $ssrg$source98)
export const main = (_unit: undefined) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source214, () => _ssrg_console_println((($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: string }): string => $ssrg_match.tag === "Just" ? (($ssrg$fallbackValue: string): string => $ssrg$fallbackValue)($ssrg_match.value) : (($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: string }): string => $ssrg_match.tag === "Just" ? (($ssrg$fallbackValue: string): string => $ssrg$fallbackValue)($ssrg_match.value) : "anonymous")(requested))(cached)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source261, () => _ssrg_console_println($ssrg$atSource($ssrg$source271, () => ((_ssrg_show_intShow["show"]((($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }): number => $ssrg_match.tag === "Just" ? (($ssrg$fallbackValue: number): number => $ssrg$fallbackValue)($ssrg_match.value) : $ssrg$atSource($ssrg$source287, () => fail(undefined), false))($ssrg$atSource($ssrg$source277, () => _ssrg_maybe_Just(7), false)))) as string), false)), true), () => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source298, () => _ssrg_console_println($ssrg$atSource($ssrg$source308, () => ((_ssrg_show_intShow["show"]((($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }): number => $ssrg_match.tag === "Just" ? (($ssrg$fallbackValue: number): number => $ssrg$fallbackValue)($ssrg_match.value) : 8)(_ssrg_maybe_Nothing))) as string), false)), true), () => $ssrg$atSource($ssrg$source330, () => _ssrg_console_println($ssrg$atSource($ssrg$source340, () => ((_ssrg_show_intShow["show"]($ssrg$atSource($ssrg$source346, () => resolve<number>($ssrg$atSource($ssrg$source355, () => _ssrg_maybe_Just(9), false))(0), false))) as string), false)), true))))
const cached: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: string } = _ssrg_maybe_Nothing;
const requested: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: string } = $ssrg$atSource($ssrg$source171, () => _ssrg_maybe_Just("request"), false);
