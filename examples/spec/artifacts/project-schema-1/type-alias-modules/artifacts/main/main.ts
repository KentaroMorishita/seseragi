import { duplicate } from "./domain.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
$ssrg$assertUnicodeVersion("17.0.0")

const swap = <A,>(value: { readonly "left": A; readonly "right": A }) => (<$ssrg_record_type,>($ssrg_record: $ssrg_record_type) => $ssrg_record as $ssrg_record_type & { readonly [K in keyof globalThis.Object as K extends keyof $ssrg_record_type ? never : K]?: never })(({ "left": (value)["right"], "right": (value)["left"] } as const))
const message = (value: { readonly "left": string; readonly "right": string }) => (value)["left"]
export const main = (_unit: undefined) => _ssrg_console_println(message(swap(duplicate<string>("Imported alias"))))
