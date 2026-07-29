import { add as _ssrg_int_add } from "@seseragi/runtime/int"

export const add = (left: number) => (right: number) => _ssrg_int_add(left, right)
export const addOne = (value: number) => add(value)(1)
export const total = (unit: undefined) => addOne(41)
