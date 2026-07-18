import { println as _ssrg_console_println } from "@seseragi/runtime/console"

type Status =
  | { readonly tag: "Ready" }
  | { readonly tag: "Waiting" };
const Ready: Status = { tag: "Ready" } as const;
const Waiting: Status = { tag: "Waiting" } as const;
export const __ssrg$instance$Eq$0 = { "eq": (left: Status) => (right: Status) => (($ssrg_match: readonly [Status, Status]): boolean => $ssrg_match[0].tag === "Ready" && $ssrg_match[1].tag === "Ready" ? true : $ssrg_match[0].tag === "Waiting" && $ssrg_match[1].tag === "Waiting" ? true : false)([left, right] as const) } as const;
const applyComparison = (compare: (argument: Status) => (argument: Status) => boolean) => (left: Status) => (right: Status) => compare(left)(right)
const applyIntComparison = (compare: (argument: bigint) => (argument: bigint) => boolean) => (left: bigint) => (right: bigint) => compare(left)(right)
const applyEquality = <T,>(compare: (argument: T) => (argument: T) => boolean) => (left: T) => (right: T) => compare(left)(right)
const genericSame = <T,>(left: T) => (right: T) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => applyEquality((_argument0) => (_argument1) => __ssrg$evidence$0["eq"](_argument0)(_argument1))(left)(right)
const render = (checks: readonly [boolean, boolean, boolean, boolean, boolean]) => (($ssrg_match: readonly [boolean, boolean, boolean, boolean, boolean]): string => $ssrg_match[0] === true && $ssrg_match[1] === true && $ssrg_match[2] === true && $ssrg_match[3] === true && $ssrg_match[4] === true ? "Equality sections: same / different" : "unexpected equality section result")(checks)
export const main = (_unit: undefined) => _ssrg_console_println(render([applyComparison((_argument0) => (_argument1) => __ssrg$instance$Eq$0["eq"](_argument0)(_argument1))(Ready)(Ready), applyComparison((_argument0) => (_argument1) => __ssrg$instance$Eq$0["eq"](_argument0)(_argument1) === false)(Ready)(Waiting), applyIntComparison((_argument0) => (_argument1) => _argument0 === _argument1)(42n)(42n), applyIntComparison((_argument0) => (_argument1) => _argument0 !== _argument1)(42n)(41n), genericSame(Waiting)(Waiting)(__ssrg$instance$Eq$0)] as const))
