import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { intOrd as _ssrg_int_ord_dictionary, intEq as _ssrg_int_eq_dictionary } from "@seseragi/runtime/equality"
import { subtract as _ssrg_int_subtract, add as _ssrg_int_add } from "@seseragi/runtime/int"
const $ssrg$source89 = { file: "project/self-tail-loop/src/main.ssrg", line: 5, column: 5, uri: "seseragi://fixture/self-tail-loop::main", start: 89, end: 126 };
const $ssrg$source96 = { file: "project/self-tail-loop/src/main.ssrg", line: 5, column: 12, uri: "seseragi://fixture/self-tail-loop::main", start: 96, end: 107 };
const $ssrg$source110 = { file: "project/self-tail-loop/src/main.ssrg", line: 5, column: 26, uri: "seseragi://fixture/self-tail-loop::main", start: 110, end: 125 };
const $ssrg$source176 = { file: "project/self-tail-loop/src/main.ssrg", line: 8, column: 3, uri: "seseragi://fixture/self-tail-loop::main", start: 176, end: 190 };
$ssrg$assertUnicodeVersion("17.0.0")

const sumTo: (argument: number) => (argument: number) => number = (current: number) => (total: number) => { let $ssrg$scalarSlot0: number = current;let $ssrg$scalarSlot1: number = total; while (true) { const current: number = $ssrg$scalarSlot0;const total: number = $ssrg$scalarSlot1; if ((_ssrg_int_ord_dictionary["compare"](current)(0))["tag"] !== "Greater") { return total; } else { const $ssrg$scalarNext0 = $ssrg$atSource($ssrg$source89, () => _ssrg_int_subtract(current, 1, $ssrg$source96), false);const $ssrg$scalarNext1 = $ssrg$atSource($ssrg$source89, () => _ssrg_int_add(total, current, $ssrg$source110), false);$ssrg$scalarSlot0 = $ssrg$scalarNext0;$ssrg$scalarSlot1 = $ssrg$scalarNext1;continue; } } }
export const selfTailLoopWorks = (unit: undefined) => $ssrg$atSource($ssrg$source176, () => _ssrg_int_eq_dictionary["eq"]($ssrg$atSource($ssrg$source176, () => sumTo(100000)(0), false))(5000050000), false)
