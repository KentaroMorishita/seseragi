import { subtract as _ssrg_int64_subtract, add as _ssrg_int64_add } from "@seseragi/runtime/int64"

const sumTo = (current: bigint) => (total: bigint) => { const $ssrg$tail = Symbol(); while (true) { const $ssrg$result = current <= 0n ? total : ({ [$ssrg$tail]: [_ssrg_int64_subtract(current, 1n), _ssrg_int64_add(total, current)] } as never); const $ssrg$tailResult: unknown = $ssrg$result; if (typeof $ssrg$tailResult === "object" && $ssrg$tailResult !== null && $ssrg$tail in $ssrg$tailResult) { const $ssrg$arguments = ($ssrg$tailResult as { readonly [$ssrg$tail]: readonly [bigint, bigint] })[$ssrg$tail]; current = $ssrg$arguments[0]; total = $ssrg$arguments[1]; continue; } return $ssrg$result; } }
export const selfTailLoopWorks = (unit: undefined) => sumTo(100000n)(0n) === 5000050000n
