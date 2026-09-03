import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { stringAdd as _ssrg_string_add_dictionary } from "@seseragi/runtime/string"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
$ssrg$assertUnicodeVersion("17.0.0")

export type Badge =
  | { readonly tag: "Active" };
export const Active: Badge = { tag: "Active" } as const;
export const __ssrg$instance$Ready$0 = { "ready": (value: Badge) => "Badge is ready" } as const;
const describe = <T,>(value: T) => (suffix: string) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => _ssrg_string_add_dictionary["add"](__ssrg$evidence$0["ready"](value))(suffix)
const applyLabel = (labeler: (argument: string) => string) => labeler("!")
const label = <T,>(value: T) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => applyLabel((((__ssrg$partial$0: string) => describe(value)(__ssrg$partial$0)(__ssrg$evidence$0)) as (argument: string) => string))
export const main = (_unit: undefined) => _ssrg_console_println(((label(Active)(__ssrg$instance$Ready$0)) as string))
