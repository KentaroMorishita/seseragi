import { add as _ssrg_int_add } from "@seseragi/runtime/int"

export const add = (x: number) => (y: number) => _ssrg_int_add(x, y)
