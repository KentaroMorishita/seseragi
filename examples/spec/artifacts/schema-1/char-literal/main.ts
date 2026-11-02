import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { Just as _ssrg_maybe_Just } from "@seseragi/runtime/sum"
import { charShow as _ssrg_show_charShow, tupleShow as _ssrg_show_tupleShow, maybeShow as _ssrg_show_maybeShow, arrayShow as _ssrg_show_arrayShow } from "@seseragi/runtime/show"
const $ssrg$source248 = { file: "main.ssrg", line: 7, column: 35, uri: "seseragi://artifact/char-literal", start: 248, end: 256 };
const $ssrg$source258 = { file: "main.ssrg", line: 7, column: 45, uri: "seseragi://artifact/char-literal", start: 258, end: 270 };
const $ssrg$source299 = { file: "main.ssrg", line: 8, column: 28, uri: "seseragi://artifact/char-literal", start: 299, end: 316 };
const $ssrg$source318 = { file: "main.ssrg", line: 8, column: 47, uri: "seseragi://artifact/char-literal", start: 318, end: 332 };
$ssrg$assertUnicodeVersion("17.0.0")

const $ssrg$closedDisplay$0 = (() => { const create = () => _ssrg_show_tupleShow<readonly [string, string]>(_ssrg_show_charShow, _ssrg_show_charShow); let value: ReturnType<typeof create> | undefined; return (): ReturnType<typeof create> => (value ??= create()); })()
const identity = <A,>(value: A) => value
const character = (value: string) => value
const classify = (value: string) => (($ssrg_match: string): number => $ssrg_match === "a" ? 1 : $ssrg_match === "λ" ? 2 : 0)(value)
const account$prime: string = "瀬";
const escaped: string = "λ";
const nested: ReadonlyArray<{ readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: string }> = [$ssrg$atSource($ssrg$source248, () => _ssrg_maybe_Just("a"), false), $ssrg$atSource($ssrg$source258, () => _ssrg_maybe_Just(escaped), false)];
const tuple: readonly [string, string] = [$ssrg$atSource($ssrg$source299, () => identity<string>(account$prime), false), $ssrg$atSource($ssrg$source318, () => character("'"), false)] as const;
export const result: string = $ssrg$closedDisplay$0()["show"](tuple) + ", " + _ssrg_show_arrayShow<{ readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: string }>(_ssrg_show_maybeShow<string>(_ssrg_show_charShow))["show"](nested) + ", " + _ssrg_show_charShow["show"]("λ");
