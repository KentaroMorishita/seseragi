import { Nothing as _ssrg_maybe_Nothing, Right as _ssrg_either_Right } from "@seseragi/runtime/sum"
import { intShow as _ssrg_show_intShow, intDebug as _ssrg_debug_intDebug, floatShow as _ssrg_show_floatShow, floatDebug as _ssrg_debug_floatDebug, neverShow as _ssrg_show_neverShow, neverDebug as _ssrg_debug_neverDebug, boolShow as _ssrg_show_boolShow, boolDebug as _ssrg_debug_boolDebug, unitShow as _ssrg_show_unitShow, unitDebug as _ssrg_debug_unitDebug, charShow as _ssrg_show_charShow, charDebug as _ssrg_debug_charDebug, stringDebug as _ssrg_debug_stringDebug, maybeShow as _ssrg_show_maybeShow, eitherDebug as _ssrg_debug_eitherDebug } from "@seseragi/runtime/show"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

const absent: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: never } = _ssrg_maybe_Nothing;
const available: { readonly tag: "Left"; readonly value: never } | { readonly tag: "Right"; readonly value: string } = _ssrg_either_Right("ready");
export const showInt = (value: bigint) => _ssrg_show_intShow["show"](value)
export const debugInt = (value: bigint) => _ssrg_debug_intDebug["debug"](value)
export const showFloat = (value: number) => _ssrg_show_floatShow["show"](value)
export const debugFloat = (value: number) => _ssrg_debug_floatDebug["debug"](value)
export const showNever = (value: never) => _ssrg_show_neverShow["show"](value)
export const debugNever = (value: never) => _ssrg_debug_neverDebug["debug"](value)
export const showBool = (value: boolean) => _ssrg_show_boolShow["show"](value)
export const debugBool = (value: boolean) => _ssrg_debug_boolDebug["debug"](value)
export const showUnit = (value: undefined) => _ssrg_show_unitShow["show"](value)
export const debugUnit = (value: undefined) => _ssrg_debug_unitDebug["debug"](value)
export const showChar = (value: string) => _ssrg_show_charShow["show"](value)
export const debugChar = (value: string) => _ssrg_debug_charDebug["debug"](value)
export const debugString = (value: string) => _ssrg_debug_stringDebug["debug"](value)
export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_debug_intDebug["debug"](42n)), () => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_show_maybeShow<never>(_ssrg_show_neverShow)["show"](absent)), () => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_debug_eitherDebug<never, string>(_ssrg_debug_neverDebug, _ssrg_debug_stringDebug)["debug"](available)), () => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_show_boolShow["show"](true)), () => _ssrg_effect_flatMap(_ssrg_console_println(_ssrg_show_unitShow["show"](undefined)), () => _ssrg_console_println(_ssrg_debug_stringDebug["debug"]("line\n\"quoted\"")))))))
