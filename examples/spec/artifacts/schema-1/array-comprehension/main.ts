import { add as _ssrg_int_add } from "@seseragi/runtime/int"
import { collectFlatMap as _ssrg_array_comprehend_flat, collectMap as _ssrg_array_comprehend } from "@seseragi/runtime/array"
import { intEq as _ssrg_int_eq_dictionary } from "@seseragi/runtime/equality"

export const selectedPairs = (unit: undefined) => _ssrg_array_comprehend_flat([1, 2], (left) => true, (left) => _ssrg_array_comprehend([10, 20], (right) => _ssrg_int_eq_dictionary["eq"](right)(20), (right) => _ssrg_int_add(left, right)))
