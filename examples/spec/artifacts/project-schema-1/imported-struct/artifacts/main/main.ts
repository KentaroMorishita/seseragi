import { type Player } from "./domain.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { stringAdd as _ssrg_string_add_dictionary } from "@seseragi/runtime/string"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source185 = { file: "project/imported-struct/src/main.ssrg", line: 8, column: 24, uri: "seseragi://fixture/imported-struct::main", start: 185, end: 211 };
const $ssrg$source240 = { file: "project/imported-struct/src/main.ssrg", line: 12, column: 3, uri: "seseragi://fixture/imported-struct::main", start: 240, end: 286 };
$ssrg$assertUnicodeVersion("17.0.0")

const improve = (player: Player) => (({ ...player, "score": 42 } as const) as unknown as Player)
const render = (player: Player) => (($ssrg_match: Player): string => ((name: string): string => $ssrg$atSource($ssrg$source185, () => _ssrg_string_add_dictionary["add"](name)(": imported struct"), false))($ssrg_match["name"]))(player)
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source240, () => _ssrg_console_println($ssrg$atSource($ssrg$source240, () => render($ssrg$atSource($ssrg$source240, () => improve((({ "name": "Mio", "score": 12 } as const) as unknown as Player)), false)), false)), true)
