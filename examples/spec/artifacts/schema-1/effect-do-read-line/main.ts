import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { readLine as _ssrg_stdin_readLine } from "@seseragi/runtime/stdin"
const $ssrg$source28 = { file: "main.ssrg", line: 2, column: 8, uri: "seseragi://artifact/effect-do-read-line", start: 28, end: 39 };
$ssrg$assertUnicodeVersion("17.0.0")

export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source28, () => _ssrg_stdin_readLine(), true)
