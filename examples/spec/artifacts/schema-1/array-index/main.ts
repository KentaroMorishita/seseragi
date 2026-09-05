import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { index as _ssrg_array_index, get as _ssrg_array_get } from "@seseragi/runtime/array"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { intShow as _ssrg_show_intShow, maybeShow as _ssrg_show_maybeShow, boolShow as _ssrg_show_boolShow } from "@seseragi/runtime/show"
import { subtract as _ssrg_int_subtract } from "@seseragi/runtime/int"
import { intEq as _ssrg_int_eq_dictionary, maybeEq as _ssrg_maybe_eq_dictionary } from "@seseragi/runtime/equality"
$ssrg$assertUnicodeVersion("17.0.0")

export const pick = <A,>(values: ReadonlyArray<A>) => (offset: number) => _ssrg_array_index(values, offset)
export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_console_println(((_ssrg_show_maybeShow<number>(_ssrg_show_intShow)["show"](_ssrg_array_index(values, 0))) as string)), () => _ssrg_effect_flatMap(_ssrg_console_println(((_ssrg_show_maybeShow<number>(_ssrg_show_intShow)["show"](_ssrg_array_index(values, 1))) as string)), () => _ssrg_effect_flatMap(_ssrg_console_println(((_ssrg_show_maybeShow<number>(_ssrg_show_intShow)["show"](_ssrg_array_index(values, 2))) as string)), () => _ssrg_effect_flatMap(_ssrg_console_println(((_ssrg_show_maybeShow<number>(_ssrg_show_intShow)["show"](_ssrg_array_index(values, _ssrg_int_subtract(0, 1)))) as string)), () => _ssrg_effect_flatMap(_ssrg_console_println(((_ssrg_show_maybeShow<number>(_ssrg_show_intShow)["show"](_ssrg_array_index(values, 3))) as string)), () => _ssrg_effect_flatMap(_ssrg_console_println(((_ssrg_show_maybeShow<number>(_ssrg_show_intShow)["show"](_ssrg_array_index(values, 99))) as string)), () => _ssrg_effect_flatMap(_ssrg_console_println(((_ssrg_show_maybeShow<number>(_ssrg_show_intShow)["show"](_ssrg_array_index(empty, 0))) as string)), () => _ssrg_effect_flatMap(_ssrg_console_println(((_ssrg_show_maybeShow<number>(_ssrg_show_intShow)["show"](read(1))) as string)), () => _ssrg_effect_flatMap(_ssrg_console_println(((_ssrg_show_maybeShow<number>(_ssrg_show_intShow)["show"](pick([7, 8])(1))) as string)), () => _ssrg_console_println(((_ssrg_show_boolShow["show"](_ssrg_maybe_eq_dictionary<number>(_ssrg_int_eq_dictionary)["eq"](_ssrg_array_index(values, 1))(_ssrg_array_get(1, values)))) as string)))))))))))
const values: ReadonlyArray<number> = [10, 20, 30];
const empty: ReadonlyArray<number> = [] as ReadonlyArray<number>;
const read: (argument: number) => { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number } = pick(values);
