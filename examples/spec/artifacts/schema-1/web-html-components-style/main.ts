import { style as _ssrg_html_style, section as _ssrg_html_section, h2 as _ssrg_html_h2, p as _ssrg_html_p, main as _ssrg_html_main, h1 as _ssrg_html_h1, renderToString as _ssrg_html_renderToString, type Style as Style, type Html as Html } from "@seseragi/runtime/html"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

type Msg =
  | { readonly tag: "Confirm" };
const Confirm: Msg = { tag: "Confirm" } as const;
const cardStyle: Style = _ssrg_html_style(({ "variables": ({ "cardShadow": "0 4px 16px #0002" } as const), "backgroundColor": "#fff", "boxShadow": "var(--card-shadow)", "borderRadius": "16px", "padding": "24px" } as const));
const card = (title: string) => (detail: string) => _ssrg_html_section(({ "className": "card", "style": cardStyle, "children": [_ssrg_html_h2(({ "style": _ssrg_html_style(({ "color": "#185f50", "marginTop": "0" } as const)), "children": title } as const)), _ssrg_html_p(({ "style": _ssrg_html_style(({ "color": "#315c53", "marginBottom": "0" } as const)), "children": detail } as const))] } as const))
const page = (_unit: undefined) => _ssrg_html_main(({ "id": "app", "style": _ssrg_html_style(({ "backgroundColor": "#eef6f3", "padding": "32px" } as const)), "children": [_ssrg_html_h1(({ "onClick": Confirm, "style": _ssrg_html_style(({ "color": "#123b32", "marginTop": "0" } as const)), "children": "Seseragi components" } as const)), card("Reusable card")("Function component from children")] } as const))
export const main = (_unit: undefined) => _ssrg_console_println(_ssrg_html_renderToString(page(undefined)))
