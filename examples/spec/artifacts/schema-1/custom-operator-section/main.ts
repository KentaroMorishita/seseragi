import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { subtract as _ssrg_int_subtract } from "@seseragi/runtime/int"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
$ssrg$assertUnicodeVersion("17.0.0")

export const __ssrg$instance$Difference$0 = { "difference": (left: number) => (right: number) => _ssrg_int_subtract(left, right) } as const;
const __ssrg$operator$3c5e3e = <A,>(left: A) => (right: A) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => ((__ssrg$evidence$0["difference"](left)(right)) as A)
const applyPair = (step: (argument: number) => (argument: number) => number) => (left: number) => (right: number) => step(left)(right)
const applyOne = (step: (argument: number) => number) => (value: number) => step(value)
const subtractFromTen = (right: number) => applyOne((((__ssrg$partial$0: number) => __ssrg$operator$3c5e3e(10)(__ssrg$partial$0)(__ssrg$instance$Difference$0)) as (argument: number) => number))(right)
export const main = (_unit: undefined) => _ssrg_console_println("Custom operator section: higher-order=" + _ssrg_show_intShow["show"](applyPair((((__ssrg$partial$0: number) => (__ssrg$partial$1: number) => __ssrg$operator$3c5e3e(__ssrg$partial$0)(__ssrg$partial$1)(__ssrg$instance$Difference$0)) as (argument: number) => (argument: number) => number))(10)(3)) + ", partial=" + _ssrg_show_intShow["show"](subtractFromTen(3)))
