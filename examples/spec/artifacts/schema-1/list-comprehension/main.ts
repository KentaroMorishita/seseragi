import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { fromArray as _ssrg_list_from_array, collectMap as _ssrg_list_comprehend, reduce as _ssrg_list_reduce, type List as List } from "@seseragi/runtime/list"
import { multiply as _ssrg_int_multiply, remainder as _ssrg_int_remainder, add as _ssrg_int_add } from "@seseragi/runtime/int"
import { intEq as _ssrg_int_eq_dictionary } from "@seseragi/runtime/equality"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
$ssrg$assertUnicodeVersion("17.0.0")

const oddSquares = (values: List<number>) => _ssrg_list_from_array(_ssrg_list_comprehend(values, (value) => _ssrg_int_eq_dictionary["eq"](_ssrg_int_remainder(value, 2))(1), (value) => _ssrg_int_multiply(value, value)))
const total = (values: List<number>) => _ssrg_list_reduce(0, (((_argument0) => (_argument1) => _ssrg_int_add(_argument0, _argument1)) as (argument: number) => (argument: number) => number), values)
const renderTotal = (total: number) => (($ssrg_match: number): string => $ssrg_match === 35 ? "persistent List total: 35" : "unexpected List total")(total)
export const main = (_unit: undefined) => _ssrg_console_println(renderTotal(total(oddSquares(_ssrg_list_from_array([1, 2, 3, 4, 5])))))
