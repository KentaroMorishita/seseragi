import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { stringShow as _ssrg_show_stringShow } from "@seseragi/runtime/show"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { Just as _ssrg_maybe_Just } from "@seseragi/runtime/sum"
const $ssrg$source287 = { file: "main.ssrg", line: 14, column: 20, uri: "seseragi://artifact/standard-show-evidence", start: 287, end: 303 };
const $ssrg$source361 = { file: "main.ssrg", line: 19, column: 3, uri: "seseragi://artifact/standard-show-evidence", start: 361, end: 373 };
const $ssrg$source398 = { file: "main.ssrg", line: 22, column: 3, uri: "seseragi://artifact/standard-show-evidence", start: 398, end: 418 };
const $ssrg$source405 = { file: "main.ssrg", line: 22, column: 10, uri: "seseragi://artifact/standard-show-evidence", start: 405, end: 417 };
$ssrg$assertUnicodeVersion("17.0.0")

export const __ssrg$instance$Render$0 = <T,>(__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => ({ "render": (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: T }) => (($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: T }): string => $ssrg_match.tag === "Nothing" ? "No value" : $ssrg_match.tag === "Just" ? ((item: T): string => $ssrg$atSource($ssrg$source287, () => ((acknowledge<T>(item)(__ssrg$evidence$0)) as string), false))($ssrg_match.value) : ((): never => { throw new globalThis.Error("non-exhaustive Seseragi match"); })())(value) }) as const;
const acknowledge = <T,>(value: T) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => "Standard Show evidence"
export const label = (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: string }) => $ssrg$atSource($ssrg$source361, () => ((__ssrg$instance$Render$0<string>(_ssrg_show_stringShow)["render"](value)) as string), false)
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source398, () => _ssrg_console_println($ssrg$atSource($ssrg$source398, () => label($ssrg$atSource($ssrg$source405, () => _ssrg_maybe_Just("ready"), false)), false)), true)
