import { add as _ssrg_int64_add, subtract as _ssrg_int64_subtract } from "@seseragi/runtime/int64"

export const localFunctionWorks = (unit: undefined) => (() => { const offset: bigint = 2n; return (() => { const identity = <A,>(value: A) => value; return (() => { const countdown = (current: bigint) => current === 0n ? identity(_ssrg_int64_add(40n, offset)) : countdown(_ssrg_int64_subtract(current, 1n)); return countdown(3n) === 42n; })(); })(); })()
