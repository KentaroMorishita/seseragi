import { add as _ssrg_int_add, multiply as _ssrg_int_multiply } from "@seseragi/runtime/int"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

const add = (offset: number) => (value: number) => _ssrg_int_add(offset, value)
const double = (value: number) => _ssrg_int_multiply(value, 2)
const describe = (value: number) => (($ssrg_match: number): string => $ssrg_match === 42 ? "Pipeline answer: 42" : "Unexpected result")(value)
export const main = (_unit: undefined) => _ssrg_console_println(describe(double(add(5)(16))))
