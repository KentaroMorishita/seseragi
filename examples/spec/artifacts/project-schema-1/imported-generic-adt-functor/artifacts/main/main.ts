import { type Box, Boxed, transform, __ssrg$instance$Functor$0 } from "./domain.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
$ssrg$assertUnicodeVersion("17.0.0")

const increment = (value: number) => _ssrg_int_add(value, 1)
const render = (value: Box<number>) => (($ssrg_match: Box<number>): string => $ssrg_match.tag === "Boxed" && $ssrg_match.value === 42 ? "Imported Box Functor: 42" : "Imported Box Functor: another value")(value)
export const main = (_unit: undefined) => _ssrg_console_println(render(((transform(increment)(Boxed(41))(__ssrg$instance$Functor$0)) as Box<number>)))
