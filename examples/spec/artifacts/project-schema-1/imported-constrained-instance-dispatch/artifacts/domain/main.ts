import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
const $ssrg$source354 = { file: "project/imported-constrained-instance-dispatch/src/domain.ssrg", line: 18, column: 20, uri: "seseragi://fixture/imported-constrained-instance-dispatch::domain", start: 354, end: 364 };
const $ssrg$source431 = { file: "project/imported-constrained-instance-dispatch/src/domain.ssrg", line: 24, column: 3, uri: "seseragi://fixture/imported-constrained-instance-dispatch::domain", start: 431, end: 444 };
$ssrg$assertUnicodeVersion("17.0.0")

export const __ssrg$instance$Ready$0 = { "ready": (value: number) => "imported constrained ready" } as const;
export const __ssrg$instance$Inspect$1 = <T,>(__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => ({ "inspect": (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: T }) => (($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: T }): string => $ssrg_match.tag === "Nothing" ? "not ready" : $ssrg_match.tag === "Just" ? ((item: T): string => $ssrg$atSource($ssrg$source354, () => ((__ssrg$evidence$0["ready"](item)) as string), false))($ssrg_match.value) : ((): never => { throw new globalThis.Error("non-exhaustive Seseragi match"); })())(value) }) as const;
export const report = <T,>(value: T) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => $ssrg$atSource($ssrg$source431, () => ((__ssrg$evidence$0["inspect"](value)) as string), false)
