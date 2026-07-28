import { boundedShow as _ssrg_show_boundedShow, stringShow as _ssrg_show_stringShow, type Show as _ssrg_show_Show } from "@seseragi/runtime/show"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

export type Badge =
  | { readonly tag: "Active" };
export const Active: Badge = { tag: "Active" } as const;
export const __ssrg$instance$Show$0: _ssrg_show_Show<Badge> = _ssrg_show_boundedShow((value: Badge): string => { switch (value.tag) { case "Active": return "Active"; } });
export const render = (名前$prime: string) => (badge: Badge) => "Hello, " + _ssrg_show_stringShow["show"](名前$prime) + ": " + __ssrg$instance$Show$0["show"](badge)
export const main = (_unit: undefined) => _ssrg_console_println(render("Seseragi")(Active))
