import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
const $ssrg$source316 = { file: "main.ssrg", line: 16, column: 5, uri: "seseragi://artifact/string-escapes", start: 316, end: 337 };
$ssrg$assertUnicodeVersion("17.0.0")

const classify = (value: string) => (($ssrg_match: string): string => $ssrg_match === "line\nnext" ? "matched" : "other")(value)
export const stringEscapeResults = (unit: undefined) => ["line\nnext", "tab\tvalue", "carriage\rreturn", "quote: \"Seseragi\"", "slash: \\", "lambda: λ", "nul:\u0000end", $ssrg$atSource($ssrg$source316, () => classify("line\nnext"), false)]
