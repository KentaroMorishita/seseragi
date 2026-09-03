import { box, __ssrg$method$Box$map as map, __ssrg$method$Box$get as get, type Box } from "./domain.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
$ssrg$assertUnicodeVersion("17.0.0")

const evaluate = (value: number) => get(map(box(value))((item: number) => _ssrg_int_add(item, item)))
export const main = (_unit: undefined) => _ssrg_console_println("Imported inherent method: " + _ssrg_show_intShow["show"](evaluate(21)))
