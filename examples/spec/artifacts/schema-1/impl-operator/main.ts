import { add as _ssrg_int_add } from "@seseragi/runtime/int"
import { intEq as _ssrg_int_eq_dictionary } from "@seseragi/runtime/equality"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

declare const __ssrg$brand$Score: unique symbol;
type Score = {
  readonly "value": number;
  readonly [__ssrg$brand$Score]: true;
};
export const __ssrg$instance$Add$0 = { "add": (self: Score) => (bonus: number) => (({ "value": _ssrg_int_add((self)["value"], bonus) } as const) as unknown as Score) } as const;
export const __ssrg$instance$Eq$1 = { "eq": (self: Score) => (other: Score) => _ssrg_int_eq_dictionary["eq"]((self)["value"])((other)["value"]) } as const;
const total = (unit: undefined) => __ssrg$instance$Add$0["add"]((({ "value": 21 } as const) as unknown as Score))(21)
const render = (score: Score) => __ssrg$instance$Eq$1["eq"](score)((({ "value": 42 } as const) as unknown as Score)) ? (($ssrg_match: Score): string => ((value: number): string => "Impl operator: " + _ssrg_show_intShow["show"](value))($ssrg_match["value"]))(score) : "unexpected score"
export const main = (_unit: undefined) => _ssrg_console_println(render(total(undefined)))
