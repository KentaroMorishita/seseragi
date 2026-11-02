import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source138 = { file: "main.ssrg", line: 9, column: 9, uri: "seseragi://artifact/newtype-user-id", start: 138, end: 144 };
const $ssrg$source254 = { file: "main.ssrg", line: 15, column: 3, uri: "seseragi://artifact/newtype-user-id", start: 254, end: 268 };
$ssrg$assertUnicodeVersion("17.0.0")

export type UserId =
  | { readonly tag: "UserId"; readonly value: number };
export const UserId = (value: number): UserId => ({ tag: "UserId", value } as const);
const raw = (id: UserId) => (($ssrg_match: UserId): number => $ssrg_match.tag === "UserId" ? ((value: number): number => value)($ssrg_match.value) : ((): never => { throw new globalThis.Error("non-exhaustive Seseragi match"); })())(id)
const render = (id: UserId) => (($ssrg_match: number): string => $ssrg_match === 42 ? "UserId keeps its nominal boundary: 42" : "unexpected UserId")($ssrg$atSource($ssrg$source138, () => raw(id), false))
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source254, () => _ssrg_console_println($ssrg$atSource($ssrg$source254, () => render($ssrg$atSource($ssrg$source254, () => UserId(42), false)), false)), true)
