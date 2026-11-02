import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { boolShow as _ssrg_show_boolShow } from "@seseragi/runtime/show"
const $ssrg$source103 = { file: "main.ssrg", line: 2, column: 50, uri: "seseragi://artifact/implicit-unit-function-own-binder", start: 103, end: 119 };
const $ssrg$source121 = { file: "main.ssrg", line: 2, column: 68, uri: "seseragi://artifact/implicit-unit-function-own-binder", start: 121, end: 127 };
const $ssrg$source151 = { file: "main.ssrg", line: 3, column: 22, uri: "seseragi://artifact/implicit-unit-function-own-binder", start: 151, end: 176 };
const $ssrg$source160 = { file: "main.ssrg", line: 3, column: 31, uri: "seseragi://artifact/implicit-unit-function-own-binder", start: 160, end: 175 };
const $ssrg$source166 = { file: "main.ssrg", line: 3, column: 37, uri: "seseragi://artifact/implicit-unit-function-own-binder", start: 166, end: 174 };
$ssrg$assertUnicodeVersion("17.0.0")

const selected = <Unit,>(_unit: undefined) => (value: Unit) => false
export const probe = (_unit: undefined) => (() => { const next: (argument: number) => boolean = $ssrg$atSource($ssrg$source103, () => selected<number>(undefined), false); return $ssrg$atSource($ssrg$source121, () => next(1), false); })()
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source151, () => _ssrg_console_println($ssrg$atSource($ssrg$source160, () => ((_ssrg_show_boolShow["show"]($ssrg$atSource($ssrg$source166, () => probe(undefined), false))) as string), false)), true)
