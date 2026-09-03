import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { section as _ssrg_html_section, button as _ssrg_html_button, p as _ssrg_html_p, type Html as Html } from "@seseragi/runtime/html"
import { type Effect as Effect } from "@seseragi/runtime/effect"
$ssrg$assertUnicodeVersion("17.0.0")

export const view = (summary: string) => (clear: Effect<{  }, never, undefined>) => _ssrg_html_section(({ "children": [_ssrg_html_button(({ "onClick": clear, "children": "Clear" } as const)), _ssrg_html_p(({ "children": summary } as const))] } as const))
