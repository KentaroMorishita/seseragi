import { subtract as _ssrg_int64_subtract } from "@seseragi/runtime/int64"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"

export const __ssrg$instance$Difference$0 = { "difference": (left: bigint) => (right: bigint) => _ssrg_int64_subtract(left, right) } as const;
const __ssrg$operator$3c5e3e = <A,>(left: A) => (right: A) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => __ssrg$evidence$0["difference"](left)(right)
const applyPair = (step: (argument: bigint) => (argument: bigint) => bigint) => (left: bigint) => (right: bigint) => step(left)(right)
const applyOne = (step: (argument: bigint) => bigint) => (value: bigint) => step(value)
const subtractFromTen = (right: bigint) => applyOne((__ssrg$partial$0: bigint) => __ssrg$operator$3c5e3e(10n)(__ssrg$partial$0)(__ssrg$instance$Difference$0))(right)
export const main = (_unit: undefined) => _ssrg_console_println("Custom operator section: higher-order=" + _ssrg_show_intShow["show"](applyPair((__ssrg$partial$0: bigint) => (__ssrg$partial$1: bigint) => __ssrg$operator$3c5e3e(__ssrg$partial$0)(__ssrg$partial$1)(__ssrg$instance$Difference$0))(10n)(3n)) + ", partial=" + _ssrg_show_intShow["show"](subtractFromTen(3n)))
