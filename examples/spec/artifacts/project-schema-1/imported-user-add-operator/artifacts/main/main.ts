import { Points, type Score, __ssrg$instance$Add$0 } from "./domain.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { reduce as _ssrg_array_reduce } from "@seseragi/runtime/array"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
$ssrg$assertUnicodeVersion("17.0.0")

const addBonus = (bonus: number) => (score: Score) => __ssrg$instance$Add$0["add"](score)(bonus)
const total = (values: ReadonlyArray<number>) => addBonus(0)(_ssrg_array_reduce(Points(0), (_argument0) => (_argument1) => __ssrg$instance$Add$0["add"](_argument0)(_argument1), values))
const render = (score: Score) => (($ssrg_match: Score): string => $ssrg_match.tag === "Points" && $ssrg_match.value === 42 ? "Imported Add: 42" : "unexpected score")(score)
export const main = (_unit: undefined) => _ssrg_console_println(render(total([10, 12, 20])))
