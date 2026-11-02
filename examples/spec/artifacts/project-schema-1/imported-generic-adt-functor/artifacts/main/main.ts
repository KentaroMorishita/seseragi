import { type Box, Boxed, transform, __ssrg$instance$Functor$0, type $ssrg$kind$Box } from "./domain.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source83 = { file: "project/imported-generic-adt-functor/src/main.ssrg", line: 3, column: 34, uri: "seseragi://fixture/imported-generic-adt-functor::main", start: 83, end: 92 };
const $ssrg$source272 = { file: "project/imported-generic-adt-functor/src/main.ssrg", line: 12, column: 3, uri: "seseragi://fixture/imported-generic-adt-functor::main", start: 272, end: 280 };
$ssrg$assertUnicodeVersion("17.0.0")

const increment = (value: number) => _ssrg_int_add(value, 1, $ssrg$source83)
const render = (value: Box<number>) => (($ssrg_match: Box<number>): string => $ssrg_match.tag === "Boxed" && $ssrg_match.value === 42 ? "Imported Box Functor: 42" : "Imported Box Functor: another value")(value)
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source272, () => _ssrg_console_println($ssrg$atSource($ssrg$source272, () => render($ssrg$atSource($ssrg$source272, () => ((transform(increment)($ssrg$atSource($ssrg$source272, () => Boxed<number>(41), false))(__ssrg$instance$Functor$0)) as Box<number>), false)), false)), true)
