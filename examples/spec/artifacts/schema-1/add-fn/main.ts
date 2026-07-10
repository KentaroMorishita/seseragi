import { add as _ssrg_int64_add } from "@seseragi/runtime/int64"

export const add = (x: bigint) => (y: bigint) => _ssrg_int64_add(x, y)
