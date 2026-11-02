import { announce } from "./effects.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
const $ssrg$source61 = { file: "project/imported-effect-console/src/main.ssrg", line: 4, column: 3, uri: "seseragi://fixture/imported-effect-console::main", start: 61, end: 77 };
$ssrg$assertUnicodeVersion("17.0.0")

export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source61, () => announce("hello"), true)
