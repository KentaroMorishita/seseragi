import { Points, type Score, __ssrg$instance$Add$0 } from "./domain.js"
import { reduce as _ssrg_array_reduce } from "@seseragi/runtime/array"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

const addBonus = (bonus: bigint) => (score: Score) => __ssrg$instance$Add$0["add"](score)(bonus)
const total = (values: ReadonlyArray<bigint>) => addBonus(0n)(_ssrg_array_reduce(Points(0n), (_argument0) => (_argument1) => __ssrg$instance$Add$0["add"](_argument0)(_argument1), values))
const render = (score: Score) => (($ssrg_match: Score): string => $ssrg_match.tag === "Points" && $ssrg_match.value === 42n ? "Imported Add: 42" : "unexpected score")(score)
export const main = (_unit: undefined) => _ssrg_console_println(render(total([10n, 12n, 20n])))
