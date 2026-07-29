import { fromArray as _ssrg_list_from_array, type List as List } from "@seseragi/runtime/list"
import { subtract as _ssrg_int_subtract, add as _ssrg_int_add } from "@seseragi/runtime/int"

export const nearby = (value: number) => _ssrg_list_from_array([_ssrg_int_subtract(value, 1), value, _ssrg_int_add(value, 1)])
export const keep = (values: List<number>) => values
