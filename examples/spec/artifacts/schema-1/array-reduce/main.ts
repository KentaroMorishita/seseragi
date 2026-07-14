import { reduce as _ssrg_array_reduce } from "@seseragi/runtime/array"
import { add as _ssrg_int64_add } from "@seseragi/runtime/int64"

export const arrayReduceWorks = (unit: undefined) => _ssrg_array_reduce(0n, (_argument0) => (_argument1) => _ssrg_int64_add(_argument0, _argument1), [4n, 8n, 15n, 16n, 23n, 42n]) === 108n
