import { add as _ssrg_int64_add } from "@seseragi/runtime/int64"
import { intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

declare const __ssrg$brand$Score: unique symbol;
type Score = {
  readonly "value": bigint;
  readonly [__ssrg$brand$Score]: true;
};
export const __ssrg$instance$Add$0 = { "add": (self: Score) => (bonus: bigint) => (({ "value": _ssrg_int64_add((self)["value"], bonus) } as const) as unknown as Score) } as const;
export const __ssrg$instance$Eq$1 = { "eq": (self: Score) => (other: Score) => (self)["value"] === (other)["value"] } as const;
const total = (unit: undefined) => __ssrg$instance$Add$0["add"]((({ "value": 21n } as const) as unknown as Score))(21n)
const render = (score: Score) => __ssrg$instance$Eq$1["eq"](score)((({ "value": 42n } as const) as unknown as Score)) ? (($ssrg_match: Score): string => ((value: bigint): string => "Impl operator: " + _ssrg_show_intShow["show"](value))($ssrg_match["value"]))(score) : "unexpected score"
export const main = (_unit: undefined) => _ssrg_console_println(render(total(undefined)))
