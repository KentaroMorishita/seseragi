import { box, __ssrg$method$Box$map as map, __ssrg$method$Box$get as get, type Box } from "./domain.js"
import { add as _ssrg_int64_add } from "@seseragi/runtime/int64"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"

const evaluate = (value: bigint) => get(map(box(value))((item: bigint) => _ssrg_int64_add(item, item)))
export const main = (_unit: undefined) => _ssrg_console_println("Imported inherent method: " + _ssrg_show_intShow["show"](evaluate(21n)))
