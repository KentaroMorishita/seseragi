import { transform, __ssrg$instance$Functor$0 } from "./domain.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { add as _ssrg_int_add } from "@seseragi/runtime/int"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { Just as _ssrg_maybe_Just } from "@seseragi/runtime/sum"
const $ssrg$source71 = { file: "project/imported-higher-order-functor/src/main.ssrg", line: 3, column: 34, uri: "seseragi://fixture/imported-higher-order-functor::main", start: 71, end: 80 };
const $ssrg$source280 = { file: "project/imported-higher-order-functor/src/main.ssrg", line: 13, column: 3, uri: "seseragi://fixture/imported-higher-order-functor::main", start: 280, end: 287 };
$ssrg$assertUnicodeVersion("17.0.0")

const increment = (value: number) => _ssrg_int_add(value, 1, $ssrg$source71)
const render = (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }) => (($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }): string => $ssrg_match.tag === "Nothing" ? "Nothing" : $ssrg_match.tag === "Just" && $ssrg_match.value === 42 ? "Imported mapper: Just 42" : "Imported mapper: another value")(value)
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source280, () => _ssrg_console_println($ssrg$atSource($ssrg$source280, () => render($ssrg$atSource($ssrg$source280, () => ((transform(increment)($ssrg$atSource($ssrg$source280, () => _ssrg_maybe_Just(41), false))(__ssrg$instance$Functor$0)) as { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }), false)), false)), true)
