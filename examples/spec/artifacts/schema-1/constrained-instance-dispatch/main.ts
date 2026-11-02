import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { Just as _ssrg_maybe_Just } from "@seseragi/runtime/sum"
const $ssrg$source406 = { file: "main.ssrg", line: 21, column: 20, uri: "seseragi://artifact/constrained-instance-dispatch", start: 406, end: 416 };
const $ssrg$source471 = { file: "main.ssrg", line: 25, column: 46, uri: "seseragi://artifact/constrained-instance-dispatch", start: 471, end: 483 };
const $ssrg$source548 = { file: "main.ssrg", line: 30, column: 3, uri: "seseragi://artifact/constrained-instance-dispatch", start: 548, end: 577 };
const $ssrg$source557 = { file: "main.ssrg", line: 30, column: 12, uri: "seseragi://artifact/constrained-instance-dispatch", start: 557, end: 576 };
const $ssrg$source564 = { file: "main.ssrg", line: 30, column: 19, uri: "seseragi://artifact/constrained-instance-dispatch", start: 564, end: 575 };
$ssrg$assertUnicodeVersion("17.0.0")

export type Badge =
  | { readonly tag: "Active" };
export const Active: Badge = { tag: "Active" } as const;
export const __ssrg$instance$Ready$0 = { "ready": (value: Badge) => "Constrained dictionary: ready" } as const;
export const __ssrg$instance$Render$1 = <T,>(__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => ({ "render": (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: T }) => (($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: T }): string => $ssrg_match.tag === "Nothing" ? "Constrained dictionary: empty" : $ssrg_match.tag === "Just" ? ((item: T): string => $ssrg$atSource($ssrg$source406, () => ((__ssrg$evidence$0["ready"](item)) as string), false))($ssrg_match.value) : ((): never => { throw new globalThis.Error("non-exhaustive Seseragi match"); })())(value) }) as const;
export const label = (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: Badge }) => $ssrg$atSource($ssrg$source471, () => ((__ssrg$instance$Render$1<Badge>(__ssrg$instance$Ready$0)["render"](value)) as string), false)
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source548, () => _ssrg_console_println($ssrg$atSource($ssrg$source557, () => label($ssrg$atSource($ssrg$source564, () => _ssrg_maybe_Just(Active), false)), false)), true)
