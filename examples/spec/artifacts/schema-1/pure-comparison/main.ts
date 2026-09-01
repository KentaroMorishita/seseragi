import { intEq as _ssrg_int_eq_dictionary, boolEq as _ssrg_bool_eq_dictionary, stringEq as _ssrg_string_eq_dictionary } from "@seseragi/runtime/equality"

export const isZero = (value: number) => _ssrg_int_eq_dictionary["eq"](value)(0)
export const sameBool = (left: boolean) => (right: boolean) => _ssrg_bool_eq_dictionary["eq"](left)(right)
export const sameString = (left: string) => (right: string) => _ssrg_string_eq_dictionary["eq"](left)(right) === false
