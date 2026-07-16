import { println as _ssrg_console_println } from "@seseragi/runtime/console"

export type UserId =
  | { readonly tag: "UserId"; readonly value: bigint };
export const UserId = (value: bigint): UserId => ({ tag: "UserId", value } as const);
const raw = (id: UserId) => (($ssrg_match: UserId): bigint => $ssrg_match.tag === "UserId" ? ((value: bigint): bigint => value)($ssrg_match.value) : ((): never => { throw new Error("non-exhaustive Seseragi match"); })())(id)
const render = (id: UserId) => (($ssrg_match: bigint): string => $ssrg_match === 42n ? "UserId keeps its nominal boundary: 42" : "unexpected UserId")(raw(id))
export const main = (_unit: undefined) => _ssrg_console_println(render(UserId(42n)))
