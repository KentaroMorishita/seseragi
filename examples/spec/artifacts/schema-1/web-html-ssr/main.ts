import { div as _ssrg_html_div, p as _ssrg_html_p, button as _ssrg_html_button, renderToString as _ssrg_html_renderToString, type Html as Html } from "@seseragi/runtime/html"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

type Action =
  | { readonly tag: "Confirm" };
const Confirm: Action = { tag: "Confirm" } as const;
const page = (_unit: undefined) => _ssrg_html_div(({ "id": "app", "className": "container", "children": [_ssrg_html_p(({ "children": "Hello <Seseragi>" } as const)), _ssrg_html_button(({ "onClick": Confirm, "children": "OK" } as const))] } as const))
export const main = (_unit: undefined) => _ssrg_console_println(_ssrg_html_renderToString(page(undefined)))
