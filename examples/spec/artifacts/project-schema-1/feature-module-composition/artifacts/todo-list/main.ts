import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { section as _ssrg_html_section, button as _ssrg_html_button, p as _ssrg_html_p, type Html as Html } from "@seseragi/runtime/html"
import { type Effect as Effect } from "@seseragi/runtime/effect"
const $ssrg$source148 = { file: "project/feature-module-composition/src/todo/list.ssrg", line: 7, column: 3, uri: "seseragi://fixture/feature-module-composition::todo/list", start: 148, end: 280 };
const $ssrg$source185 = { file: "project/feature-module-composition/src/todo/list.ssrg", line: 9, column: 7, uri: "seseragi://fixture/feature-module-composition::todo/list", start: 185, end: 234 };
const $ssrg$source242 = { file: "project/feature-module-composition/src/todo/list.ssrg", line: 10, column: 7, uri: "seseragi://fixture/feature-module-composition::todo/list", start: 242, end: 270 };
$ssrg$assertUnicodeVersion("17.0.0")

export const view = (summary: string) => (clear: Effect<{  }, never, undefined>) => $ssrg$atSource($ssrg$source148, () => _ssrg_html_section((<$ssrg_record_type,>($ssrg_record: $ssrg_record_type) => $ssrg_record as $ssrg_record_type & { readonly [K in keyof globalThis.Object as K extends keyof $ssrg_record_type ? never : K]?: never })(({ "children": [$ssrg$atSource($ssrg$source185, () => _ssrg_html_button((<$ssrg_record_type,>($ssrg_record: $ssrg_record_type) => $ssrg_record as $ssrg_record_type & { readonly [K in keyof globalThis.Object as K extends keyof $ssrg_record_type ? never : K]?: never })(({ "onClick": clear, "children": "Clear" } as const))), false), $ssrg$atSource($ssrg$source242, () => _ssrg_html_p((<$ssrg_record_type,>($ssrg_record: $ssrg_record_type) => $ssrg_record as $ssrg_record_type & { readonly [K in keyof globalThis.Object as K extends keyof $ssrg_record_type ? never : K]?: never })(({ "children": summary } as const))), false)] } as const))), false)
