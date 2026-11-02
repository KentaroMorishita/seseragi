import { Points, type Score, __ssrg$instance$Add$0 } from "./domain.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { reduce as _ssrg_array_reduce } from "@seseragi/runtime/array"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source94 = { file: "project/imported-user-add-operator/src/main.ssrg", line: 4, column: 3, uri: "seseragi://fixture/imported-user-add-operator::main", start: 94, end: 107 };
const $ssrg$source150 = { file: "project/imported-user-add-operator/src/main.ssrg", line: 7, column: 3, uri: "seseragi://fixture/imported-user-add-operator::main", start: 150, end: 183 };
const $ssrg$source170 = { file: "project/imported-user-add-operator/src/main.ssrg", line: 8, column: 14, uri: "seseragi://fixture/imported-user-add-operator::main", start: 170, end: 178 };
const $ssrg$source344 = { file: "project/imported-user-add-operator/src/main.ssrg", line: 18, column: 3, uri: "seseragi://fixture/imported-user-add-operator::main", start: 344, end: 367 };
$ssrg$assertUnicodeVersion("17.0.0")

const addBonus = (bonus: number) => (score: Score) => $ssrg$atSource($ssrg$source94, () => __ssrg$instance$Add$0["add"](score)(bonus), false)
const total = (values: ReadonlyArray<number>) => $ssrg$atSource($ssrg$source150, () => addBonus(0)($ssrg$atSource($ssrg$source150, () => _ssrg_array_reduce($ssrg$atSource($ssrg$source170, () => Points(0), false), ((__ssrg$instance$Add$0["add"]) as (argument: Score) => (argument: number) => Score), values), false)), false)
const render = (score: Score) => (($ssrg_match: Score): string => $ssrg_match.tag === "Points" && $ssrg_match.value === 42 ? "Imported Add: 42" : "unexpected score")(score)
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source344, () => _ssrg_console_println($ssrg$atSource($ssrg$source344, () => render($ssrg$atSource($ssrg$source344, () => total([10, 12, 20]), false)), false)), true)
