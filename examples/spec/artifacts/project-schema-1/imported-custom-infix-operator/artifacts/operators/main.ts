import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { subtract as _ssrg_int_subtract } from "@seseragi/runtime/int"
const $ssrg$source63 = { file: "project/imported-custom-infix-operator/src/operators.ssrg", line: 3, column: 3, uri: "seseragi://fixture/imported-custom-infix-operator::operators", start: 63, end: 75 };
$ssrg$assertUnicodeVersion("17.0.0")

export const __ssrg$operator$3c5e3e = (left: number) => (right: number) => _ssrg_int_subtract(left, right, $ssrg$source63)
