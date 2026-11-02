import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
import { intEq as _ssrg_int_eq_dictionary } from "@seseragi/runtime/equality"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source107 = { file: "main.ssrg", line: 7, column: 20, uri: "seseragi://artifact/impl-operator", start: 107, end: 125 };
const $ssrg$source178 = { file: "main.ssrg", line: 10, column: 5, uri: "seseragi://artifact/impl-operator", start: 178, end: 203 };
const $ssrg$source240 = { file: "main.ssrg", line: 14, column: 3, uri: "seseragi://artifact/impl-operator", start: 240, end: 264 };
const $ssrg$source306 = { file: "main.ssrg", line: 17, column: 6, uri: "seseragi://artifact/impl-operator", start: 306, end: 334 };
const $ssrg$source509 = { file: "main.ssrg", line: 27, column: 3, uri: "seseragi://artifact/impl-operator", start: 509, end: 536 };
const $ssrg$source519 = { file: "main.ssrg", line: 27, column: 13, uri: "seseragi://artifact/impl-operator", start: 519, end: 536 };
const $ssrg$source528 = { file: "main.ssrg", line: 27, column: 22, uri: "seseragi://artifact/impl-operator", start: 528, end: 536 };
$ssrg$assertUnicodeVersion("17.0.0")

declare const __ssrg$brand$Score: unique symbol;
type Score = {
  readonly "value": number;
  readonly [__ssrg$brand$Score]: true;
};
export const __ssrg$instance$Add$0 = { "add": (self: Score) => (bonus: number) => (({ "value": _ssrg_int_add((self)["value"], bonus, $ssrg$source107) } as const) as unknown as Score) } as const;
export const __ssrg$instance$Eq$1 = { "eq": (self: Score) => (other: Score) => $ssrg$atSource($ssrg$source178, () => _ssrg_int_eq_dictionary["eq"]((self)["value"])((other)["value"]), false) } as const;
const total = (unit: undefined) => $ssrg$atSource($ssrg$source240, () => __ssrg$instance$Add$0["add"]((({ "value": 21 } as const) as unknown as Score))(21), false)
const render = (score: Score) => $ssrg$atSource($ssrg$source306, () => __ssrg$instance$Eq$1["eq"](score)((({ "value": 42 } as const) as unknown as Score)), false) ? (($ssrg_match: Score): string => ((value: number): string => "Impl operator: " + _ssrg_show_intShow["show"](value))($ssrg_match["value"]))(score) : "unexpected score"
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source509, () => _ssrg_console_println($ssrg$atSource($ssrg$source519, () => render($ssrg$atSource($ssrg$source528, () => total(undefined), false)), false)), true)
