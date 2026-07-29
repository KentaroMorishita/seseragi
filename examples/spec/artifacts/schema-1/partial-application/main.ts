import { add as _ssrg_int_add } from "@seseragi/runtime/int"

export const add = (left: number) => (right: number) => _ssrg_int_add(left, right)
export const addTo = (value: number) => add(value)
