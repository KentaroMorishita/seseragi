import { type Badge, renderImported, __ssrg$instance$Render$2, __ssrg$instance$Debug$1, __ssrg$instance$Show$0 } from "./domain.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { arrayShow as _ssrg_show_arrayShow, maybeDebug as _ssrg_debug_maybeDebug } from "@seseragi/runtime/show"
const $ssrg$source99 = { file: "project/imported-show-debug-collection-evidence/src/main.ssrg", line: 4, column: 3, uri: "seseragi://fixture/imported-show-debug-collection-evidence::main", start: 99, end: 110 };
const $ssrg$source161 = { file: "project/imported-show-debug-collection-evidence/src/main.ssrg", line: 7, column: 3, uri: "seseragi://fixture/imported-show-debug-collection-evidence::main", start: 161, end: 172 };
const $ssrg$source237 = { file: "project/imported-show-debug-collection-evidence/src/main.ssrg", line: 10, column: 3, uri: "seseragi://fixture/imported-show-debug-collection-evidence::main", start: 237, end: 257 };
$ssrg$assertUnicodeVersion("17.0.0")

export const render = (values: ReadonlyArray<Badge>) => $ssrg$atSource($ssrg$source99, () => ((_ssrg_show_arrayShow<Badge>(__ssrg$instance$Show$0)["show"](values)) as string), false)
export const inspect = (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: Badge }) => $ssrg$atSource($ssrg$source161, () => ((_ssrg_debug_maybeDebug<Badge>(__ssrg$instance$Debug$1)["debug"](value)) as string), false)
export const renderThroughImported = (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: Badge }) => $ssrg$atSource($ssrg$source237, () => ((renderImported<{ readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: Badge }>(value)(__ssrg$instance$Render$2<Badge>(_ssrg_show_arrayShow<Badge>(__ssrg$instance$Show$0)))) as string), false)
