import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { intEq as _ssrg_int_eq_dictionary } from "@seseragi/runtime/equality"
import { reduce as _ssrg_array_reduce } from "@seseragi/runtime/array"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
const $ssrg$source47 = { file: "main.ssrg", line: 2, column: 3, uri: "seseragi://artifact/array-reduce", start: 47, end: 82 };
$ssrg$assertUnicodeVersion("17.0.0")

export const arrayReduceWorks = (unit: undefined) => $ssrg$atSource($ssrg$source47, () => _ssrg_int_eq_dictionary["eq"]($ssrg$atSource($ssrg$source47, () => _ssrg_array_reduce(0, (((_argument0) => (_argument1) => _ssrg_int_add(_argument0, _argument1)) as (argument: number) => (argument: number) => number), [4, 8, 15, 16, 23, 42]), false))(108), false)
