import { duplicate } from "./domain.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
$ssrg$assertUnicodeVersion("17.0.0")

const swap = <A,>(value: { readonly "left": A; readonly "right": A }) => ({ "left": (value)["right"], "right": (value)["left"] } as const)
const message = (value: { readonly "left": string; readonly "right": string }) => (value)["left"]
export const main = (_unit: undefined) => _ssrg_console_println(message(swap(duplicate("Imported alias"))))
