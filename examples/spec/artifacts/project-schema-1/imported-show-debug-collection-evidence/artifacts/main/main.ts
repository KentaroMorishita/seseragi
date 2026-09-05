import { type Badge, renderImported, __ssrg$instance$Render$2, __ssrg$instance$Debug$1, __ssrg$instance$Show$0 } from "./domain.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { arrayShow as _ssrg_show_arrayShow, maybeDebug as _ssrg_debug_maybeDebug } from "@seseragi/runtime/show"
$ssrg$assertUnicodeVersion("17.0.0")

export const render = (values: ReadonlyArray<Badge>) => ((_ssrg_show_arrayShow<Badge>(__ssrg$instance$Show$0)["show"](values)) as string)
export const inspect = (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: Badge }) => ((_ssrg_debug_maybeDebug<Badge>(__ssrg$instance$Debug$1)["debug"](value)) as string)
export const renderThroughImported = (value: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: Badge }) => ((renderImported(value)(__ssrg$instance$Render$2<Badge>(_ssrg_show_arrayShow<Badge>(__ssrg$instance$Show$0)))) as string)
