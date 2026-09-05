import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { Just as _ssrg_maybe_Just } from "@seseragi/runtime/sum"
$ssrg$assertUnicodeVersion("17.0.0")

export const __ssrg$instance$Tag$0 = <T,>() => ({ "tag": (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: T }) => "Generic dictionary: maybe" }) as const;
export const label = (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: number }) => ((__ssrg$instance$Tag$0<number>()["tag"](value)) as string)
export const main = (_unit: undefined) => _ssrg_console_println(label(_ssrg_maybe_Just(1)))
