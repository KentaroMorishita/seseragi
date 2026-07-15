import { add as _ssrg_int64_add, multiply as _ssrg_int64_multiply } from "@seseragi/runtime/int64"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

const add = (offset: bigint) => (value: bigint) => _ssrg_int64_add(offset, value)
const double = (value: bigint) => _ssrg_int64_multiply(value, 2n)
const describe = (value: bigint) => (($ssrg_match: bigint): string => $ssrg_match === 42n ? "Pipeline answer: 42" : "Unexpected result")(value)
export const main = (_unit: undefined) => _ssrg_console_println(describe(double(add(5n)(16n))))
