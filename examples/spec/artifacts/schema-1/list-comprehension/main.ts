import { fromArray as _ssrg_list_from_array, collectMap as _ssrg_list_comprehend, reduce as _ssrg_list_reduce, type List as List } from "@seseragi/runtime/list"
import { multiply as _ssrg_int64_multiply, remainder as _ssrg_int64_remainder, add as _ssrg_int64_add } from "@seseragi/runtime/int64"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

const oddSquares = (values: List<bigint>) => _ssrg_list_from_array(_ssrg_list_comprehend(values, (value) => _ssrg_int64_remainder(value, 2n) === 1n, (value) => _ssrg_int64_multiply(value, value)))
const total = (values: List<bigint>) => _ssrg_list_reduce(0n, (_argument0) => (_argument1) => _ssrg_int64_add(_argument0, _argument1), values)
const renderTotal = (total: bigint) => (($ssrg_match: bigint): string => $ssrg_match === 35n ? "persistent List total: 35" : "unexpected List total")(total)
export const main = (_unit: undefined) => _ssrg_console_println(renderTotal(total(oddSquares(_ssrg_list_from_array([1n, 2n, 3n, 4n, 5n])))))
