import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { Just as _ssrg_maybe_Just } from "@seseragi/runtime/sum"
import { charShow as _ssrg_show_charShow, tupleShow as _ssrg_show_tupleShow, maybeShow as _ssrg_show_maybeShow, arrayShow as _ssrg_show_arrayShow } from "@seseragi/runtime/show"
$ssrg$assertUnicodeVersion("17.0.0")

const identity = <A,>(value: A) => value
const character = (value: string) => value
const classify = (value: string) => (($ssrg_match: string): number => $ssrg_match === "a" ? 1 : $ssrg_match === "λ" ? 2 : 0)(value)
const account$prime: string = "瀬";
const escaped: string = "λ";
const nested: ReadonlyArray<{ readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: string }> = [_ssrg_maybe_Just("a"), _ssrg_maybe_Just(escaped)];
const tuple: readonly [string, string] = [identity(account$prime), character("'")] as const;
export const result: string = _ssrg_show_tupleShow<readonly [string, string]>(_ssrg_show_charShow, _ssrg_show_charShow)["show"](tuple) + ", " + _ssrg_show_arrayShow<{ readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: string }>(_ssrg_show_maybeShow<string>(_ssrg_show_charShow))["show"](nested) + ", " + _ssrg_show_charShow["show"]("λ");
