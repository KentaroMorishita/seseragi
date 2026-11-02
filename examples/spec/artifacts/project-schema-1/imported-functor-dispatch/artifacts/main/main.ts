import { incrementAll, __ssrg$instance$Functor$0 } from "./domain.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { Just as _ssrg_maybe_Just } from "@seseragi/runtime/sum"
const $ssrg$source241 = { file: "project/imported-functor-dispatch/src/main.ssrg", line: 11, column: 3, uri: "seseragi://fixture/imported-functor-dispatch::main", start: 241, end: 248 };
$ssrg$assertUnicodeVersion("17.0.0")

const render = (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }) => (($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }): string => $ssrg_match.tag === "Nothing" ? "Nothing" : $ssrg_match.tag === "Just" && $ssrg_match.value === 42 ? "Imported Functor: Just 42" : "Imported Functor: another value")(value)
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source241, () => _ssrg_console_println($ssrg$atSource($ssrg$source241, () => render($ssrg$atSource($ssrg$source241, () => ((incrementAll($ssrg$atSource($ssrg$source241, () => _ssrg_maybe_Just(41), false))(__ssrg$instance$Functor$0)) as { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }), false)), false)), true)
