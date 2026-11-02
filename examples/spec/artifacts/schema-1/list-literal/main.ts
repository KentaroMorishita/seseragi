import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { fromArray as _ssrg_list_from_array, type List as List } from "@seseragi/runtime/list"
import { subtract as _ssrg_int_subtract, add as _ssrg_int_add } from "@seseragi/runtime/int"
const $ssrg$source44 = { file: "main.ssrg", line: 2, column: 5, uri: "seseragi://artifact/list-literal", start: 44, end: 53 };
const $ssrg$source62 = { file: "main.ssrg", line: 2, column: 23, uri: "seseragi://artifact/list-literal", start: 62, end: 71 };
$ssrg$assertUnicodeVersion("17.0.0")

export const nearby = (value: number) => _ssrg_list_from_array([_ssrg_int_subtract(value, 1, $ssrg$source44), value, _ssrg_int_add(value, 1, $ssrg$source62)])
export const keep = (values: List<number>) => values
