import { start } from "./app.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
const $ssrg$source52 = { file: "project/feature-module-lifetime/src/main.ssrg", line: 3, column: 22, uri: "seseragi://fixture/feature-module-lifetime::main", start: 52, end: 60 };
$ssrg$assertUnicodeVersion("17.0.0")

export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source52, () => start(undefined), true)
