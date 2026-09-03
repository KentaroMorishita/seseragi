import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { stringShow as _ssrg_show_stringShow, intShow as _ssrg_show_intShow } from "@seseragi/runtime/show"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
$ssrg$assertUnicodeVersion("17.0.0")

declare const __ssrg$brand$Player: unique symbol;
export type Player = {
  readonly "name": string;
  readonly "score": number;
  readonly [__ssrg$brand$Player]: true;
};
const improve = (player: Player) => (({ ...player, "score": 42 } as const) as unknown as Player)
const render = (player: Player) => (($ssrg_match: Player): string => $ssrg_match["score"] === 42 ? ((name: string): string => _ssrg_show_stringShow["show"](name) + ": perfect " + _ssrg_show_intShow["show"]((player)["score"]))($ssrg_match["name"]) : ((name: string, score: number): string => _ssrg_show_stringShow["show"](name) + ": keep training at " + _ssrg_show_intShow["show"](score))($ssrg_match["name"], $ssrg_match["score"]))(player)
export const main = (_unit: undefined) => _ssrg_console_println(render(improve((({ "name": "Mio", "score": 12 } as const) as unknown as Player))))
