import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { subtract as _ssrg_int_subtract } from "@seseragi/runtime/int"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
const $ssrg$source251 = { file: "main.ssrg", line: 8, column: 5, uri: "seseragi://artifact/custom-operator-section", start: 251, end: 263 };
const $ssrg$source343 = { file: "main.ssrg", line: 14, column: 3, uri: "seseragi://artifact/custom-operator-section", start: 343, end: 364 };
const $ssrg$source451 = { file: "main.ssrg", line: 21, column: 3, uri: "seseragi://artifact/custom-operator-section", start: 451, end: 466 };
const $ssrg$source530 = { file: "main.ssrg", line: 27, column: 3, uri: "seseragi://artifact/custom-operator-section", start: 530, end: 540 };
const $ssrg$source583 = { file: "main.ssrg", line: 30, column: 3, uri: "seseragi://artifact/custom-operator-section", start: 583, end: 608 };
const $ssrg$source593 = { file: "main.ssrg", line: 30, column: 13, uri: "seseragi://artifact/custom-operator-section", start: 593, end: 601 };
const $ssrg$source673 = { file: "main.ssrg", line: 35, column: 3, uri: "seseragi://artifact/custom-operator-section", start: 673, end: 774 };
const $ssrg$source722 = { file: "main.ssrg", line: 35, column: 52, uri: "seseragi://artifact/custom-operator-section", start: 722, end: 742 };
const $ssrg$source733 = { file: "main.ssrg", line: 35, column: 63, uri: "seseragi://artifact/custom-operator-section", start: 733, end: 736 };
const $ssrg$source755 = { file: "main.ssrg", line: 35, column: 85, uri: "seseragi://artifact/custom-operator-section", start: 755, end: 772 };
$ssrg$assertUnicodeVersion("17.0.0")

export const __ssrg$instance$Difference$0 = { "difference": (left: number) => (right: number) => _ssrg_int_subtract(left, right, $ssrg$source251) } as const;
const __ssrg$operator$3c5e3e = <A,>(left: A) => (right: A) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => $ssrg$atSource($ssrg$source343, () => ((__ssrg$evidence$0["difference"](left)(right)) as A), false)
const applyPair = (step: (argument: number) => (argument: number) => number) => (left: number) => (right: number) => $ssrg$atSource($ssrg$source451, () => step(left)(right), false)
const applyOne = (step: (argument: number) => number) => (value: number) => $ssrg$atSource($ssrg$source530, () => step(value), false)
const subtractFromTen = (right: number) => $ssrg$atSource($ssrg$source583, () => applyOne($ssrg$atSource($ssrg$source593, () => (((() => { const __ssrg$partial$stage: (argument: number) => (argument: unknown) => number = ((__ssrg$operator$3c5e3e<number>(10)) as (argument: number) => (argument: unknown) => number); return (__ssrg$partial$0: number) => __ssrg$partial$stage(__ssrg$partial$0)(__ssrg$instance$Difference$0); })()) as (argument: number) => number), false))(right), false)
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source673, () => _ssrg_console_println("Custom operator section: higher-order=" + _ssrg_show_intShow["show"]($ssrg$atSource($ssrg$source722, () => applyPair((__ssrg$partial$0: number) => (__ssrg$partial$1: number) => $ssrg$atSource($ssrg$source733, () => ((__ssrg$operator$3c5e3e<number>(__ssrg$partial$0)(__ssrg$partial$1)(__ssrg$instance$Difference$0)) as number), false))(10)(3), false)) + ", partial=" + _ssrg_show_intShow["show"]($ssrg$atSource($ssrg$source755, () => subtractFromTen(3), false))), true)
