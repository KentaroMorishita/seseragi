import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { readLine as _ssrg_stdin_readLine } from "@seseragi/runtime/stdin"
const $ssrg$source25 = { file: "main.ssrg", line: 1, column: 26, uri: "seseragi://artifact/effect-compact-read-line", start: 25, end: 36 };
$ssrg$assertUnicodeVersion("17.0.0")

export const nextLine = (_unit: undefined) => $ssrg$atSource($ssrg$source25, () => _ssrg_stdin_readLine(), true)
