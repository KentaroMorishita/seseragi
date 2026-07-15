export type Badge =
  | { readonly tag: "Active" };
export const Active: Badge = { tag: "Active" } as const;
export const __ssrg$instance$Render$0 = { "render": (value: Badge) => "active" } as const;
export const status = (unit: undefined) => "ready"
