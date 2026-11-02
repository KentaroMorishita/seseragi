import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { Just as _ssrg_maybe_Just } from "@seseragi/runtime/sum"
const $ssrg$source189 = { file: "main.ssrg", line: 9, column: 44, uri: "seseragi://artifact/generic-instance-dispatch", start: 189, end: 198 };
const $ssrg$source263 = { file: "main.ssrg", line: 14, column: 3, uri: "seseragi://artifact/generic-instance-dispatch", start: 263, end: 287 };
const $ssrg$source272 = { file: "main.ssrg", line: 14, column: 12, uri: "seseragi://artifact/generic-instance-dispatch", start: 272, end: 286 };
const $ssrg$source279 = { file: "main.ssrg", line: 14, column: 19, uri: "seseragi://artifact/generic-instance-dispatch", start: 279, end: 285 };
$ssrg$assertUnicodeVersion("17.0.0")

export const __ssrg$instance$Tag$0 = <T,>() => ({ "tag": (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: T }) => "Generic dictionary: maybe" }) as const;
export const label = (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }) => $ssrg$atSource($ssrg$source189, () => ((__ssrg$instance$Tag$0<number>()["tag"](value)) as string), false)
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source263, () => _ssrg_console_println($ssrg$atSource($ssrg$source272, () => label($ssrg$atSource($ssrg$source279, () => _ssrg_maybe_Just(1), false)), false)), true)
