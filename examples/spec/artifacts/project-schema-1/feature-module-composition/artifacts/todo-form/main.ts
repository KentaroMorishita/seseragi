import { form as _ssrg_html_form, input as _ssrg_html_input, button as _ssrg_html_button, type InputEvent as InputEvent, type Html as Html } from "@seseragi/runtime/html"
import { type Effect as Effect } from "@seseragi/runtime/effect"

export const view = (draft: string) => (onDraft: (argument: InputEvent) => Effect<{  }, never, undefined>) => (onSubmit: Effect<{  }, never, undefined>) => _ssrg_html_form(({ "onSubmit": onSubmit, "children": [_ssrg_html_input(({ "id": "todo-draft", "value": draft, "placeholder": "Add a task", "onInput": onDraft } as const)), _ssrg_html_button(({ "buttonType": "submit", "children": "Add" } as const))] } as const))
