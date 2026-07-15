import { describe } from "./domain.js"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

export type Badge =
  | { readonly tag: "Active" };
export const Active: Badge = { tag: "Active" } as const;
export const __ssrg$instance$Ready$0 = { "ready": (value: Badge) => "imported ready" } as const;
export const main = (_unit: undefined) => _ssrg_console_println(describe(Active)(__ssrg$instance$Ready$0))
