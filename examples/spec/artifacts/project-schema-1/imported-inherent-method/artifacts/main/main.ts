import { box, __ssrg$method$Box$map as map, __ssrg$method$Box$get as get, type Box, type $ssrg$kind$Box } from "./domain.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
const $ssrg$source66 = { file: "project/imported-inherent-method/src/main.ssrg", line: 4, column: 3, uri: "seseragi://fixture/imported-inherent-method::main", start: 66, end: 110 };
const $ssrg$source67 = { file: "project/imported-inherent-method/src/main.ssrg", line: 4, column: 4, uri: "seseragi://fixture/imported-inherent-method::main", start: 67, end: 105 };
const $ssrg$source68 = { file: "project/imported-inherent-method/src/main.ssrg", line: 4, column: 5, uri: "seseragi://fixture/imported-inherent-method::main", start: 68, end: 77 };
const $ssrg$source93 = { file: "project/imported-inherent-method/src/main.ssrg", line: 4, column: 30, uri: "seseragi://fixture/imported-inherent-method::main", start: 93, end: 104 };
const $ssrg$source175 = { file: "project/imported-inherent-method/src/main.ssrg", line: 9, column: 3, uri: "seseragi://fixture/imported-inherent-method::main", start: 175, end: 227 };
const $ssrg$source214 = { file: "project/imported-inherent-method/src/main.ssrg", line: 9, column: 42, uri: "seseragi://fixture/imported-inherent-method::main", start: 214, end: 225 };
$ssrg$assertUnicodeVersion("17.0.0")

const evaluate = (value: number) => $ssrg$atSource($ssrg$source66, () => get<number>($ssrg$atSource($ssrg$source67, () => map<number>($ssrg$atSource($ssrg$source68, () => box<number>(value), false))((item: number) => _ssrg_int_add(item, item, $ssrg$source93)), false)), false)
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source175, () => _ssrg_console_println("Imported inherent method: " + _ssrg_show_intShow["show"]($ssrg$atSource($ssrg$source214, () => evaluate(21), false))), true)
