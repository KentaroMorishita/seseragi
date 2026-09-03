import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
$ssrg$assertUnicodeVersion("17.0.0")

export type UserId =
  | { readonly tag: "UserId"; readonly value: number };
export const UserId = (value: number): UserId => ({ tag: "UserId", value } as const);
const raw = (id: UserId) => (($ssrg_match: UserId): number => $ssrg_match.tag === "UserId" ? ((value: number): number => value)($ssrg_match.value) : ((): never => { throw new Error("non-exhaustive Seseragi match"); })())(id)
const render = (id: UserId) => (($ssrg_match: number): string => $ssrg_match === 42 ? "UserId keeps its nominal boundary: 42" : "unexpected UserId")(raw(id))
export const main = (_unit: undefined) => _ssrg_console_println(render(UserId(42)))
