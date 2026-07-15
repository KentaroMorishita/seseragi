import { println as _ssrg_console_println } from "@seseragi/runtime/console"

const invitation = (guest: string) => (event: string) => "Hello, " + guest + "! Welcome to " + event + "."
export const main = (_unit: undefined) => _ssrg_console_println(invitation("Mio")("Seseragi Night"))
