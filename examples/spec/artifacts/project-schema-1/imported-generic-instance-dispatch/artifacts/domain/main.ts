import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
const $ssrg$source221 = { file: "project/imported-generic-instance-dispatch/src/domain.ssrg", line: 11, column: 3, uri: "seseragi://fixture/imported-generic-instance-dispatch::domain", start: 221, end: 234 };
$ssrg$assertUnicodeVersion("17.0.0")

export const __ssrg$instance$Inspect$0 = <T,>() => ({ "inspect": (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: T }) => "imported generic dictionary" }) as const;
export const report = <T,>(value: T) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => $ssrg$atSource($ssrg$source221, () => ((__ssrg$evidence$0["inspect"](value)) as string), false)
