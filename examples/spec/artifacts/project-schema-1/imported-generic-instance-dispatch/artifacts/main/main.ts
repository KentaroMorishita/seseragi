import { report, __ssrg$instance$Inspect$0 } from "./domain.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { Just as _ssrg_maybe_Just } from "@seseragi/runtime/sum"
const $ssrg$source58 = { file: "project/imported-generic-instance-dispatch/src/main.ssrg", line: 4, column: 3, uri: "seseragi://fixture/imported-generic-instance-dispatch::main", start: 58, end: 74 };
const $ssrg$source66 = { file: "project/imported-generic-instance-dispatch/src/main.ssrg", line: 4, column: 11, uri: "seseragi://fixture/imported-generic-instance-dispatch::main", start: 66, end: 73 };
$ssrg$assertUnicodeVersion("17.0.0")

export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source58, () => _ssrg_console_println($ssrg$atSource($ssrg$source58, () => ((report<{ readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }>($ssrg$atSource($ssrg$source66, () => _ssrg_maybe_Just(42), false))(__ssrg$instance$Inspect$0<number>())) as string), false)), true)
