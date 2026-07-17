import { stringShow as _ssrg_show_stringShow } from "@seseragi/runtime/show"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

declare const __ssrg$brand$Player: unique symbol;
export type Player = {
  readonly "name": string;
  readonly "score": bigint;
  readonly [__ssrg$brand$Player]: true;
};
const improve = (player: Player) => (({ ...player, "score": 42n } as const) as unknown as Player)
const render = (player: Player) => (($ssrg_match: Player): string => $ssrg_match["score"] === 42n ? ((name: string): string => _ssrg_show_stringShow["show"](name) + ": perfect 42")($ssrg_match["name"]) : ((name: string): string => _ssrg_show_stringShow["show"](name) + ": keep training")($ssrg_match["name"]))(player)
export const main = (_unit: undefined) => _ssrg_console_println(render(improve((({ "name": "Mio", "score": 12n } as const) as unknown as Player))))
