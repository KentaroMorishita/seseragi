import { intEq as _ssrg_int_eq_dictionary } from "@seseragi/runtime/equality"

export const classify = (value: number) => _ssrg_int_eq_dictionary["eq"](value)(0) ? "zero" : "other"
