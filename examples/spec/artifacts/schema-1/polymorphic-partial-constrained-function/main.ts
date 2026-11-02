import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { stringAdd as _ssrg_string_add_dictionary } from "@seseragi/runtime/string"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source240 = { file: "main.ssrg", line: 17, column: 3, uri: "seseragi://artifact/polymorphic-partial-constrained-function", start: 240, end: 251 };
const $ssrg$source322 = { file: "main.ssrg", line: 22, column: 3, uri: "seseragi://artifact/polymorphic-partial-constrained-function", start: 322, end: 333 };
const $ssrg$source389 = { file: "main.ssrg", line: 28, column: 3, uri: "seseragi://artifact/polymorphic-partial-constrained-function", start: 389, end: 416 };
const $ssrg$source401 = { file: "main.ssrg", line: 28, column: 15, uri: "seseragi://artifact/polymorphic-partial-constrained-function", start: 401, end: 415 };
const $ssrg$source441 = { file: "main.ssrg", line: 31, column: 3, uri: "seseragi://artifact/polymorphic-partial-constrained-function", start: 441, end: 458 };
$ssrg$assertUnicodeVersion("17.0.0")

export type Badge =
  | { readonly tag: "Active" };
export const Active: Badge = { tag: "Active" } as const;
export const __ssrg$instance$Ready$0 = { "ready": (value: Badge) => "Badge is ready" } as const;
const describe = <T,>(value: T) => (suffix: string) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => $ssrg$atSource($ssrg$source240, () => _ssrg_string_add_dictionary["add"]($ssrg$atSource($ssrg$source240, () => ((__ssrg$evidence$0["ready"](value)) as string), false))(suffix), false)
const applyLabel = (labeler: (argument: string) => string) => $ssrg$atSource($ssrg$source322, () => labeler("!"), false)
const label = <T,>(value: T) => (__ssrg$evidence$0: Readonly<Record<string, (...args: any[]) => any>>) => $ssrg$atSource($ssrg$source389, () => applyLabel($ssrg$atSource($ssrg$source401, () => (((() => { const __ssrg$partial$stage: (argument: string) => (argument: unknown) => string = ((describe<T>(value)) as (argument: string) => (argument: unknown) => string); return (__ssrg$partial$0: string) => __ssrg$partial$stage(__ssrg$partial$0)(__ssrg$evidence$0); })()) as (argument: string) => string), false)), false)
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source441, () => _ssrg_console_println($ssrg$atSource($ssrg$source441, () => ((label<Badge>(Active)(__ssrg$instance$Ready$0)) as string), false)), true)
