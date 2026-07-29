import { add as _ssrg_int_add } from "@seseragi/runtime/int"

export const apply = <A, B,>(f: (argument: A) => B) => (value: A) => f(value)
export const increment = (value: number) => _ssrg_int_add(value, 1)
export const example = (value: number) => apply(increment)(value)
