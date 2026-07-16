import { Points, type Score, __ssrg$instance$Add$0 } from "./domain.js"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

const combine = <T,>(left: T) => (right: T) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => __ssrg$evidence$0["add"](left)(right)
const render = (score: Score) => (($ssrg_match: Score): string => $ssrg_match.tag === "Points" && $ssrg_match.value === 42n ? "Imported Add: 42" : "unexpected score")(score)
export const main = (_unit: undefined) => _ssrg_console_println(render(combine(Points(22n))(Points(20n))(__ssrg$instance$Add$0)))
