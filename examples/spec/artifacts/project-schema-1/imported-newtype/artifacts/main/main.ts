import { UserId } from "./domain.js"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

const raw = (id: UserId) => (($ssrg_match: UserId): number => $ssrg_match.tag === "UserId" ? ((value: number): number => value)($ssrg_match.value) : ((): never => { throw new Error("non-exhaustive Seseragi match"); })())(id)
const render = (id: UserId) => (($ssrg_match: number): string => $ssrg_match === 42 ? "Imported newtype: 42" : "unexpected imported newtype")(raw(id))
export const main = (_unit: undefined) => _ssrg_console_println(render(UserId(42)))
