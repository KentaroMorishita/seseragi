import { fromArray as _ssrg_list_from_array, type List as List } from "@seseragi/runtime/list"
import { subtract as _ssrg_int64_subtract, add as _ssrg_int64_add } from "@seseragi/runtime/int64"

export const nearby = (value: bigint) => _ssrg_list_from_array([_ssrg_int64_subtract(value, 1n), value, _ssrg_int64_add(value, 1n)])
export const keep = (values: List<bigint>) => values
