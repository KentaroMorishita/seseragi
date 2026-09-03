import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { intEq as _ssrg_int_eq_dictionary } from "@seseragi/runtime/equality"
import { reduce as _ssrg_array_reduce } from "@seseragi/runtime/array"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
$ssrg$assertUnicodeVersion("17.0.0")

export const arrayReduceWorks = (unit: undefined) => _ssrg_int_eq_dictionary["eq"](_ssrg_array_reduce(0, (_argument0) => (_argument1) => _ssrg_int_add(_argument0, _argument1), [4, 8, 15, 16, 23, 42]))(108)
