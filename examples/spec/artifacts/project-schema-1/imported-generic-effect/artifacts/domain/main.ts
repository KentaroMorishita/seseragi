import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source93 = { file: "project/imported-generic-effect/src/domain.ssrg", line: 5, column: 3, uri: "seseragi://fixture/imported-generic-effect::domain", start: 93, end: 113 };
const $ssrg$source102 = { file: "project/imported-generic-effect/src/domain.ssrg", line: 5, column: 12, uri: "seseragi://fixture/imported-generic-effect::domain", start: 102, end: 112 };
$ssrg$assertUnicodeVersion("17.0.0")

export const announce = <A,>(value: A) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => $ssrg$atSource($ssrg$source93, () => _ssrg_console_println($ssrg$atSource($ssrg$source102, () => ((__ssrg$evidence$0["show"](value)) as string), false)), true)
