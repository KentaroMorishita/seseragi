import { type Player } from "./domain.js"
import "./domain.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { stringAdd as _ssrg_string_add_dictionary } from "@seseragi/runtime/string"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
$ssrg$assertUnicodeVersion("17.0.0")

const improve = (player: Player) => (({ ...player, "score": 42 } as const) as unknown as Player)
const render = (player: Player) => (($ssrg_match: Player): string => ((name: string): string => _ssrg_string_add_dictionary["add"](name)(": imported struct"))($ssrg_match["name"]))(player)
export const main = (_unit: undefined) => _ssrg_console_println(render(improve((({ "name": "Mio", "score": 12 } as const) as unknown as Player))))
