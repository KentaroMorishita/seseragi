import { form as _ssrg_html_form, label as _ssrg_html_label, input as _ssrg_html_input, textarea as _ssrg_html_textarea, button as _ssrg_html_button, type InputEvent as InputEvent, type ChangeEvent as ChangeEvent, type Html as Html } from "@seseragi/runtime/html"

type Msg =
  | { readonly tag: "DraftChanged"; readonly value: string }
  | { readonly tag: "CheckedChanged"; readonly value: boolean }
  | { readonly tag: "Submitted" };
const DraftChanged = (value: string): Msg => ({ tag: "DraftChanged", value } as const);
const CheckedChanged = (value: boolean): Msg => ({ tag: "CheckedChanged", value } as const);
const Submitted: Msg = { tag: "Submitted" } as const;
const draftMessage = (event: InputEvent) => DraftChanged((event)["value"])
const checkedMessage = (event: ChangeEvent) => CheckedChanged((event)["checked"])
export const view = (draft: string) => (checked: boolean) => _ssrg_html_form(({ "onSubmit": Submitted, "children": [_ssrg_html_label(({ "htmlFor": "draft", "children": "Draft" } as const)), _ssrg_html_input(({ "id": "draft", "name": "draft", "value": draft, "required": true, "placeholder": "Type a task", "inputType": "text", "onInput": draftMessage } as const)), _ssrg_html_textarea(({ "name": "notes", "value": "", "onInput": draftMessage } as const)), _ssrg_html_input(({ "checked": checked, "inputType": "checkbox", "onChange": checkedMessage } as const)), _ssrg_html_button(({ "buttonType": "submit", "disabled": draft === "", "children": "Add" } as const))] } as const))
