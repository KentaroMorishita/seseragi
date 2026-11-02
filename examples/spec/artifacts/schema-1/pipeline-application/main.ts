import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { add as _ssrg_int_add, multiply as _ssrg_int_multiply } from "@seseragi/runtime/int"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source215 = { file: "main.ssrg", line: 3, column: 43, uri: "seseragi://artifact/pipeline-application", start: 215, end: 229 };
const $ssrg$source261 = { file: "main.ssrg", line: 5, column: 31, uri: "seseragi://artifact/pipeline-application", start: 261, end: 270 };
const $ssrg$source452 = { file: "main.ssrg", line: 16, column: 3, uri: "seseragi://artifact/pipeline-application", start: 452, end: 511 };
const $ssrg$source466 = { file: "main.ssrg", line: 17, column: 5, uri: "seseragi://artifact/pipeline-application", start: 466, end: 481 };
$ssrg$assertUnicodeVersion("17.0.0")

const add = (offset: number) => (value: number) => _ssrg_int_add(offset, value, $ssrg$source215)
const double = (value: number) => _ssrg_int_multiply(value, 2, $ssrg$source261)
const describe = (value: number) => (($ssrg_match: number): string => $ssrg_match === 42 ? "Pipeline answer: 42" : "Unexpected result")(value)
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source452, () => _ssrg_console_println($ssrg$atSource($ssrg$source466, () => describe($ssrg$atSource($ssrg$source466, () => double($ssrg$atSource($ssrg$source466, () => add(5)(16), false)), false)), false)), true)
