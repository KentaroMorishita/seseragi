import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { DispatchPreventDefault as _ssrg_html_DispatchPreventDefault, DispatchStopPropagation as _ssrg_html_DispatchStopPropagation, DispatchPreventDefaultAndStop as _ssrg_html_DispatchPreventDefaultAndStop, div as _ssrg_html_div, type MouseEvent as MouseEvent, type EventAction as EventAction, type PointerEvent as PointerEvent, type ScrollEvent as ScrollEvent, type Html as Html } from "@seseragi/runtime/html"
$ssrg$assertUnicodeVersion("17.0.0")

type Action =
  | { readonly tag: "Clicked" }
  | { readonly tag: "MouseButton"; readonly value: number }
  | { readonly tag: "PointerKind"; readonly value: string }
  | { readonly tag: "Scrolled" };
const Clicked: Action = { tag: "Clicked" } as const;
const MouseButton = (value: number): Action => ({ tag: "MouseButton", value } as const);
const PointerKind = (value: string): Action => ({ tag: "PointerKind", value } as const);
const Scrolled: Action = { tag: "Scrolled" } as const;
const mouseAction = (event: MouseEvent) => _ssrg_html_DispatchPreventDefault(MouseButton((event)["button"]))
const pointerAction = (event: PointerEvent) => _ssrg_html_DispatchStopPropagation(PointerKind((event)["pointerType"]))
const scrollAction = (event: ScrollEvent) => _ssrg_html_DispatchPreventDefaultAndStop(Scrolled)
export const view = (_unit: undefined) => _ssrg_html_div(({ "onClick": Clicked, "preventClickDefault": true, "stopClickPropagation": true, "onMouseDown": mouseAction, "onMouseUp": mouseAction, "onPointerDown": pointerAction, "onPointerUp": pointerAction, "onDoubleClick": mouseAction, "onContextMenu": mouseAction, "onScroll": scrollAction, "children": "Pointer target" } as const))
