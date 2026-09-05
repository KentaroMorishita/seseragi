import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
$ssrg$assertUnicodeVersion("17.0.0")

export type Badge =
  | { readonly tag: "Active" };
export const Active: Badge = { tag: "Active" } as const;
export const __ssrg$instance$Ready$0 = { "ready": (value: Badge) => "Badge is ready" } as const;
export const describe = <T,>(value: T) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => ((__ssrg$evidence$0["ready"](value)) as string)
export const main = (_unit: undefined) => _ssrg_console_println(((describe(Active)(__ssrg$instance$Ready$0)) as string))
