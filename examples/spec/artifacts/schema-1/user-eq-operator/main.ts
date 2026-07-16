import { println as _ssrg_console_println } from "@seseragi/runtime/console"

export type Status =
  | { readonly tag: "Ready" }
  | { readonly tag: "Waiting" };
export const Ready: Status = { tag: "Ready" } as const;
export const Waiting: Status = { tag: "Waiting" } as const;
export const __ssrg$instance$Eq$0 = { "eq": (left: Status) => (right: Status) => (($ssrg_match: readonly [Status, Status]): boolean => $ssrg_match[0].tag === "Ready" && $ssrg_match[1].tag === "Ready" ? true : $ssrg_match[0].tag === "Waiting" && $ssrg_match[1].tag === "Waiting" ? true : false)([left, right] as const) } as const;
const same = (left: Status) => (right: Status) => __ssrg$instance$Eq$0["eq"](left)(right)
const different = <T,>(left: T) => (right: T) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => __ssrg$evidence$0["eq"](left)(right) === false
const render = (checks: readonly [boolean, boolean]) => (($ssrg_match: readonly [boolean, boolean]): string => $ssrg_match[0] === true && $ssrg_match[1] === true ? "User Eq: same / different" : "unexpected Eq result")(checks)
export const main = (_unit: undefined) => _ssrg_console_println(render([same(Ready)(Ready), different(Ready)(Waiting)(__ssrg$instance$Eq$0)] as const))
