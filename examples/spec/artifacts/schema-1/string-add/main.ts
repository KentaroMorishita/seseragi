import { stringAdd as _ssrg_string_add_dictionary } from "@seseragi/runtime/string"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

const invitation = (guest: string) => (event: string) => _ssrg_string_add_dictionary["add"](_ssrg_string_add_dictionary["add"](_ssrg_string_add_dictionary["add"](_ssrg_string_add_dictionary["add"]("Hello, ")(guest))("! Welcome to "))(event))(".")
export const main = (_unit: undefined) => _ssrg_console_println(invitation("Mio")("Seseragi Night"))
