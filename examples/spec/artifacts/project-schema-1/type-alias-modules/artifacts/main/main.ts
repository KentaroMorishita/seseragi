import { duplicate } from "./domain.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source247 = { file: "project/type-alias-modules/src/main.ssrg", line: 10, column: 3, uri: "seseragi://fixture/type-alias-modules::main", start: 247, end: 300 };
const $ssrg$source257 = { file: "project/type-alias-modules/src/main.ssrg", line: 10, column: 13, uri: "seseragi://fixture/type-alias-modules::main", start: 257, end: 300 };
const $ssrg$source267 = { file: "project/type-alias-modules/src/main.ssrg", line: 10, column: 23, uri: "seseragi://fixture/type-alias-modules::main", start: 267, end: 300 };
const $ssrg$source274 = { file: "project/type-alias-modules/src/main.ssrg", line: 10, column: 30, uri: "seseragi://fixture/type-alias-modules::main", start: 274, end: 300 };
$ssrg$assertUnicodeVersion("17.0.0")

const swap = <A,>(value: { readonly "left": A; readonly "right": A }) => (<$ssrg_record_type,>($ssrg_record: $ssrg_record_type) => $ssrg_record as $ssrg_record_type & { readonly [K in keyof globalThis.Object as K extends keyof $ssrg_record_type ? never : K]?: never })(({ "left": (value)["right"], "right": (value)["left"] } as const))
const message = (value: { readonly "left": string; readonly "right": string }) => (value)["left"]
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source247, () => _ssrg_console_println($ssrg$atSource($ssrg$source257, () => message($ssrg$atSource($ssrg$source267, () => swap<string>($ssrg$atSource($ssrg$source274, () => duplicate<string>("Imported alias"), false)), false)), false)), true)
