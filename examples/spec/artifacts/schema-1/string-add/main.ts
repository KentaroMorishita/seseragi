import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { stringAdd as _ssrg_string_add_dictionary } from "@seseragi/runtime/string"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source147 = { file: "main.ssrg", line: 3, column: 3, uri: "seseragi://artifact/string-add", start: 147, end: 164 };
const $ssrg$source261 = { file: "main.ssrg", line: 8, column: 3, uri: "seseragi://artifact/string-add", start: 261, end: 304 };
const $ssrg$source271 = { file: "main.ssrg", line: 8, column: 13, uri: "seseragi://artifact/string-add", start: 271, end: 304 };
$ssrg$assertUnicodeVersion("17.0.0")

const invitation = (guest: string) => (event: string) => $ssrg$atSource($ssrg$source147, () => _ssrg_string_add_dictionary["add"]($ssrg$atSource($ssrg$source147, () => _ssrg_string_add_dictionary["add"]($ssrg$atSource($ssrg$source147, () => _ssrg_string_add_dictionary["add"]($ssrg$atSource($ssrg$source147, () => _ssrg_string_add_dictionary["add"]("Hello, ")(guest), false))("! Welcome to "), false))(event), false))("."), false)
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source261, () => _ssrg_console_println($ssrg$atSource($ssrg$source271, () => invitation("Mio")("Seseragi Night"), false)), true)
