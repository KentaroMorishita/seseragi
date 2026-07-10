import { add as _ssrg_int64_add } from "@seseragi/runtime/int64"

export const add = (left: bigint) => (right: bigint) => _ssrg_int64_add(left, right)
export const addTo = (value: bigint) => add(value)
