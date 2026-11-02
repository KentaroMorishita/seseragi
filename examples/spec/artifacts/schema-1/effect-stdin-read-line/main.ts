import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { flatMap as _ssrg_effect_flatMap, succeed as _ssrg_effect_succeed } from "@seseragi/runtime/effect"
import { readLine as _ssrg_stdin_readLine } from "@seseragi/runtime/stdin"
const $ssrg$source41 = { file: "main.ssrg", line: 3, column: 14, uri: "seseragi://artifact/effect-stdin-read-line", start: 41, end: 52 };
const $ssrg$source67 = { file: "main.ssrg", line: 4, column: 15, uri: "seseragi://artifact/effect-stdin-read-line", start: 67, end: 78 };
const $ssrg$source83 = { file: "main.ssrg", line: 5, column: 5, uri: "seseragi://artifact/effect-stdin-read-line", start: 83, end: 93 };
$ssrg$assertUnicodeVersion("17.0.0")

export const main = (_unit: undefined) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source41, () => _ssrg_stdin_readLine(), true), (first: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: string }) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source67, () => _ssrg_stdin_readLine(), true), (second: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: string }) => $ssrg$atSource($ssrg$source83, () => _ssrg_effect_succeed(undefined), true)))
