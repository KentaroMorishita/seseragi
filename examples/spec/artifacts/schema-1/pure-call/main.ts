import { add as _ssrg_int64_add } from "@seseragi/runtime/int64"

export const add = (left: bigint) => (right: bigint) => _ssrg_int64_add(left, right)
export const addOne = (value: bigint) => add(value)(1n)
export const total = (unit: undefined) => addOne(41n)
