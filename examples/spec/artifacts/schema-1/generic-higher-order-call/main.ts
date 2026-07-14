import { add as _ssrg_int64_add } from "@seseragi/runtime/int64"

export const apply = <A, B,>(f: (argument: A) => B) => (value: A) => f(value)
export const increment = (value: bigint) => _ssrg_int64_add(value, 1n)
export const example = (value: bigint) => apply(increment)(value)
