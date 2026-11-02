import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { Dispatch as _ssrg_html_Dispatch, IgnoreEvent as _ssrg_html_IgnoreEvent, button as _ssrg_html_button, type KeyboardEvent as KeyboardEvent, type EventAction as EventAction, type Html as Html } from "@seseragi/runtime/html"
import { type Effect as Effect } from "@seseragi/runtime/effect"
const $ssrg$source185 = { file: "main.ssrg", line: 10, column: 3, uri: "seseragi://artifact/web-keyboard-events", start: 185, end: 262 };
const $ssrg$source241 = { file: "main.ssrg", line: 10, column: 59, uri: "seseragi://artifact/web-keyboard-events", start: 241, end: 261 };
const $ssrg$source359 = { file: "main.ssrg", line: 13, column: 3, uri: "seseragi://artifact/web-keyboard-events", start: 359, end: 379 };
const $ssrg$source507 = { file: "main.ssrg", line: 19, column: 3, uri: "seseragi://artifact/web-keyboard-events", start: 507, end: 676 };
const $ssrg$source742 = { file: "main.ssrg", line: 29, column: 3, uri: "seseragi://artifact/web-keyboard-events", start: 742, end: 841 };
const $ssrg$source792 = { file: "main.ssrg", line: 31, column: 16, uri: "seseragi://artifact/web-keyboard-events", start: 792, end: 806 };
$ssrg$assertUnicodeVersion("17.0.0")

type Action =
  | { readonly tag: "Focused" }
  | { readonly tag: "Blurred" }
  | { readonly tag: "KeyPressed"; readonly value: string }
  | { readonly tag: "ControlKey" };
const Focused: Action = { tag: "Focused" } as const;
const Blurred: Action = { tag: "Blurred" } as const;
const KeyPressed = (value: string): Action => ({ tag: "KeyPressed", value } as const);
const ControlKey: Action = { tag: "ControlKey" } as const;
const keyAction = (event: KeyboardEvent) => $ssrg$atSource($ssrg$source185, () => _ssrg_html_Dispatch((event)["controlKey"] ? ControlKey : $ssrg$atSource($ssrg$source241, () => KeyPressed((event)["key"]), false)), false)
const taskKey = (action: Effect<{  }, never, undefined>) => (event: KeyboardEvent) => $ssrg$atSource($ssrg$source359, () => _ssrg_html_Dispatch(action), false)
const ignoreKey = (event: KeyboardEvent) => _ssrg_html_IgnoreEvent
export const view = (_unit: undefined) => $ssrg$atSource($ssrg$source507, () => _ssrg_html_button((<$ssrg_record_type,>($ssrg_record: $ssrg_record_type) => $ssrg_record as $ssrg_record_type & { readonly [K in keyof globalThis.Object as K extends keyof $ssrg_record_type ? never : K]?: never })(({ "id": "keyboard-target", "onFocus": Focused, "onBlur": Blurred, "onKeyDown": keyAction, "onKeyUp": ignoreKey, "children": "Keyboard target" } as const))), false)
export const taskView = (action: Effect<{  }, never, undefined>) => $ssrg$atSource($ssrg$source742, () => _ssrg_html_button((<$ssrg_record_type,>($ssrg_record: $ssrg_record_type) => $ssrg_record as $ssrg_record_type & { readonly [K in keyof globalThis.Object as K extends keyof $ssrg_record_type ? never : K]?: never })(({ "onFocus": action, "onKeyDown": $ssrg$atSource($ssrg$source792, () => taskKey(action), false), "children": "Effect target" } as const))), false)
