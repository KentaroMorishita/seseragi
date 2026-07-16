export type Status =
  | { readonly tag: "Ready" }
  | { readonly tag: "Waiting" };
export const Ready: Status = { tag: "Ready" } as const;
export const Waiting: Status = { tag: "Waiting" } as const;
export const __ssrg$instance$Eq$0 = { "eq": (left: Status) => (right: Status) => (($ssrg_match: readonly [Status, Status]): boolean => $ssrg_match[0].tag === "Ready" && $ssrg_match[1].tag === "Ready" ? true : $ssrg_match[0].tag === "Waiting" && $ssrg_match[1].tag === "Waiting" ? true : false)([left, right] as const) } as const;
