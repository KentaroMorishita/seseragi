import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { form as _ssrg_html_form, label as _ssrg_html_label, input as _ssrg_html_input, textarea as _ssrg_html_textarea, select as _ssrg_html_select, option as _ssrg_html_option, button as _ssrg_html_button, type InputEvent as InputEvent, type ChangeEvent as ChangeEvent, type Html as Html } from "@seseragi/runtime/html"
import { stringEq as _ssrg_string_eq_dictionary } from "@seseragi/runtime/equality"
$ssrg$assertUnicodeVersion("17.0.0")

type Action =
  | { readonly tag: "DraftChanged"; readonly value: string }
  | { readonly tag: "CheckedChanged"; readonly value: boolean }
  | { readonly tag: "Submitted" }
  | { readonly tag: "Unchanged" };
const DraftChanged = (value: string): Action => ({ tag: "DraftChanged", value } as const);
const CheckedChanged = (value: boolean): Action => ({ tag: "CheckedChanged", value } as const);
const Submitted: Action = { tag: "Submitted" } as const;
const Unchanged: Action = { tag: "Unchanged" } as const;
const draftAction = (event: InputEvent) => DraftChanged((event)["value"])
const checkedAction = (event: ChangeEvent) => (($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: boolean }): Action => $ssrg_match.tag === "Just" ? ((checked: boolean): Action => CheckedChanged(checked))($ssrg_match.value) : Unchanged)((event)["checked"])
export const view = (draft: string) => (checked: boolean) => _ssrg_html_form(({ "onSubmit": Submitted, "children": [_ssrg_html_label(({ "htmlFor": "draft", "children": "Draft" } as const)), _ssrg_html_input(({ "id": "draft", "name": "draft", "value": draft, "required": true, "placeholder": "Type a task", "inputType": "text", "onInput": draftAction, "onChange": (event: ChangeEvent) => DraftChanged((event)["value"]) } as const)), _ssrg_html_textarea(({ "name": "notes", "value": "", "onInput": draftAction, "onChange": (event: ChangeEvent) => DraftChanged((event)["value"]) } as const)), _ssrg_html_select(({ "onChange": (event: ChangeEvent) => DraftChanged((event)["value"]), "children": _ssrg_html_option(({ "value": "choice", "children": "Choice" } as const)) } as const)), _ssrg_html_input(({ "checked": checked, "inputType": "checkbox", "onChange": checkedAction } as const)), _ssrg_html_button(({ "buttonType": "submit", "disabled": _ssrg_string_eq_dictionary["eq"](draft)(""), "children": "Add" } as const))] } as const))
