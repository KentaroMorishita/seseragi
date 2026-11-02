import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
const $ssrg$source91 = { file: "main.ssrg", line: 6, column: 3, uri: "seseragi://artifact/operator-reference", start: 91, end: 109 };
const $ssrg$source164 = { file: "main.ssrg", line: 9, column: 3, uri: "seseragi://artifact/operator-reference", start: 164, end: 190 };
$ssrg$assertUnicodeVersion("17.0.0")

export const foldPair = (step: (argument: number) => (argument: number) => number) => (initial: number) => (value: number) => $ssrg$atSource($ssrg$source91, () => step(initial)(value), false)
export const addPair = (initial: number) => (value: number) => $ssrg$atSource($ssrg$source164, () => foldPair((((_argument0) => (_argument1) => _ssrg_int_add(_argument0, _argument1)) as (argument: number) => (argument: number) => number))(initial)(value), false)
