import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source275 = { file: "main.ssrg", line: 15, column: 3, uri: "seseragi://artifact/user-eq-operator", start: 275, end: 288 };
const $ssrg$source533 = { file: "main.ssrg", line: 28, column: 3, uri: "seseragi://artifact/user-eq-operator", start: 533, end: 588 };
const $ssrg$source534 = { file: "main.ssrg", line: 28, column: 4, uri: "seseragi://artifact/user-eq-operator", start: 534, end: 550 };
const $ssrg$source552 = { file: "main.ssrg", line: 28, column: 22, uri: "seseragi://artifact/user-eq-operator", start: 552, end: 575 };
$ssrg$assertUnicodeVersion("17.0.0")

export type Status =
  | { readonly tag: "Ready" }
  | { readonly tag: "Waiting" };
export const Ready: Status = { tag: "Ready" } as const;
export const Waiting: Status = { tag: "Waiting" } as const;
export const __ssrg$instance$Eq$0 = { "eq": (left: Status) => (right: Status) => (($ssrg_match: readonly [Status, Status]): boolean => $ssrg_match[0].tag === "Ready" && $ssrg_match[1].tag === "Ready" ? true : $ssrg_match[0].tag === "Waiting" && $ssrg_match[1].tag === "Waiting" ? true : false)([left, right] as const) } as const;
const same = (left: Status) => (right: Status) => $ssrg$atSource($ssrg$source275, () => __ssrg$instance$Eq$0["eq"](left)(right), false)
const different = <T,>(left: T) => (right: T) => (__ssrg$evidence$0: import("@seseragi/runtime/equality").Eq<T>) => __ssrg$evidence$0["eq"](left)(right) === false
const render = (checks: readonly [boolean, boolean]) => (($ssrg_match: readonly [boolean, boolean]): string => $ssrg_match[0] === true && $ssrg_match[1] === true ? "User Eq: same / different" : "unexpected Eq result")(checks)
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source533, () => _ssrg_console_println($ssrg$atSource($ssrg$source533, () => render([$ssrg$atSource($ssrg$source534, () => same(Ready)(Ready), false), $ssrg$atSource($ssrg$source552, () => ((different<Status>(Ready)(Waiting)(__ssrg$instance$Eq$0)) as boolean), false)] as const), false)), true)
