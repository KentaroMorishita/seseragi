import { active, describe } from "./facade.js"
import { __ssrg$instance$Ready$0, type Badge } from "./provider.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source108 = { file: "project/transitive-instance-dispatch/src/main.ssrg", line: 6, column: 3, uri: "seseragi://fixture/transitive-instance-dispatch::main", start: 108, end: 138 };
const $ssrg$source117 = { file: "project/transitive-instance-dispatch/src/main.ssrg", line: 6, column: 12, uri: "seseragi://fixture/transitive-instance-dispatch::main", start: 117, end: 137 };
const $ssrg$source127 = { file: "project/transitive-instance-dispatch/src/main.ssrg", line: 6, column: 22, uri: "seseragi://fixture/transitive-instance-dispatch::main", start: 127, end: 136 };
$ssrg$assertUnicodeVersion("17.0.0")

export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source108, () => _ssrg_console_println($ssrg$atSource($ssrg$source117, () => ((describe<Badge>($ssrg$atSource($ssrg$source127, () => active(undefined), false))(__ssrg$instance$Ready$0)) as string), false)), true)
