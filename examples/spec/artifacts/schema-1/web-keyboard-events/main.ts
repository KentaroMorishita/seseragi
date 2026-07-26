import { button as _ssrg_html_button, type KeyboardEvent as KeyboardEvent, type Html as Html } from "@seseragi/runtime/html"
import { type Effect as Effect } from "@seseragi/runtime/effect"

type Action =
  | { readonly tag: "Focused" }
  | { readonly tag: "Blurred" }
  | { readonly tag: "KeyPressed"; readonly value: string }
  | { readonly tag: "ControlKey" };
const Focused: Action = { tag: "Focused" } as const;
const Blurred: Action = { tag: "Blurred" } as const;
const KeyPressed = (value: string): Action => ({ tag: "KeyPressed", value } as const);
const ControlKey: Action = { tag: "ControlKey" } as const;
const keyAction = (event: KeyboardEvent) => (event)["controlKey"] ? ControlKey : KeyPressed((event)["key"])
const taskKey = (action: Effect<{  }, never, undefined>) => (event: KeyboardEvent) => action
export const view = (_unit: undefined) => _ssrg_html_button(({ "id": "keyboard-target", "onFocus": Focused, "onBlur": Blurred, "onKeyDown": keyAction, "onKeyUp": keyAction, "children": "Keyboard target" } as const))
export const taskView = (action: Effect<{  }, never, undefined>) => _ssrg_html_button(({ "onFocus": action, "onKeyDown": taskKey(action), "children": "Effect target" } as const))
