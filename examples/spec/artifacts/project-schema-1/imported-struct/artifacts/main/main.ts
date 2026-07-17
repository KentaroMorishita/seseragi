import { type Player } from "./domain.js"
import "./domain.js"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

const improve = (player: Player) => (({ ...player, "score": 42n } as const) as unknown as Player)
const render = (player: Player) => (($ssrg_match: Player): string => ((name: string): string => name + ": imported struct")($ssrg_match["name"]))(player)
export const main = (_unit: undefined) => _ssrg_console_println(render(improve((({ "name": "Mio", "score": 12n } as const) as unknown as Player))))
