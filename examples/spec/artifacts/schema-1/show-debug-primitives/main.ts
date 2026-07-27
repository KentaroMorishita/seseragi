import { boolShow as _ssrg_show_boolShow, boolDebug as _ssrg_debug_boolDebug, unitShow as _ssrg_show_unitShow, unitDebug as _ssrg_debug_unitDebug, charShow as _ssrg_show_charShow, charDebug as _ssrg_debug_charDebug, stringDebug as _ssrg_debug_stringDebug } from "@seseragi/runtime/show"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

export const showBool = (value: boolean) => _ssrg_show_boolShow["show"](value)
export const debugBool = (value: boolean) => _ssrg_debug_boolDebug["debug"](value)
export const showUnit = (value: undefined) => _ssrg_show_unitShow["show"](value)
export const debugUnit = (value: undefined) => _ssrg_debug_unitDebug["debug"](value)
export const showChar = (value: string) => _ssrg_show_charShow["show"](value)
export const debugChar = (value: string) => _ssrg_debug_charDebug["debug"](value)
export const debugString = (value: string) => _ssrg_debug_stringDebug["debug"](value)
export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_show_boolShow["show"](true)), () => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_show_unitShow["show"](undefined)), () => _ssrg_console_println(_ssrg_debug_stringDebug["debug"]("line\n\"quoted\""))))
