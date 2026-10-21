import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { boolShow as _ssrg_show_boolShow } from "@seseragi/runtime/show"
$ssrg$assertUnicodeVersion("17.0.0")

const selected = <Unit,>(_unit: undefined) => (value: Unit) => false
export const probe = (_unit: undefined) => (() => { const next: (argument: number) => boolean = selected(undefined); return next(1); })()
export const main = (_unit: undefined) => _ssrg_console_println(((_ssrg_show_boolShow["show"](probe(undefined))) as string))
