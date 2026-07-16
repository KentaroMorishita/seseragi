import { Ready, Waiting, __ssrg$instance$Eq$0 } from "./domain.js"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

const same = <T,>(left: T) => (right: T) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => __ssrg$evidence$0["eq"](left)(right)
const different = <T,>(left: T) => (right: T) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => __ssrg$evidence$0["eq"](left)(right) === false
const render = (checks: readonly [boolean, boolean]) => (($ssrg_match: readonly [boolean, boolean]): string => $ssrg_match[0] === true && $ssrg_match[1] === true ? "Imported Eq: same / different" : "unexpected imported Eq result")(checks)
export const main = (_unit: undefined) => _ssrg_console_println(render([same(Ready)(Ready)(__ssrg$instance$Eq$0), different(Ready)(Waiting)(__ssrg$instance$Eq$0)] as const))
