import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { boundedShow as _ssrg_show_boundedShow, stringShow as _ssrg_show_stringShow, type Show as _ssrg_show_Show } from "@seseragi/runtime/show"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source157 = { file: "main.ssrg", line: 8, column: 3, uri: "seseragi://artifact/template-interpolation", start: 157, end: 181 };
$ssrg$assertUnicodeVersion("17.0.0")

export type Badge =
  | { readonly tag: "Active" };
export const Active: Badge = { tag: "Active" } as const;
export const __ssrg$instance$Show$0: _ssrg_show_Show<Badge> = _ssrg_show_boundedShow((value: Badge): string => { switch (value.tag) { case "Active": return "Active"; } });
export const render = (名前$prime: string) => (badge: Badge) => "Hello, " + _ssrg_show_stringShow["show"](名前$prime) + ": " + __ssrg$instance$Show$0["show"](badge)
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source157, () => _ssrg_console_println($ssrg$atSource($ssrg$source157, () => render("Seseragi")(Active), false)), true)
