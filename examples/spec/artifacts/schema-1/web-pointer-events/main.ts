import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { DispatchPreventDefault as _ssrg_html_DispatchPreventDefault, DispatchStopPropagation as _ssrg_html_DispatchStopPropagation, DispatchPreventDefaultAndStop as _ssrg_html_DispatchPreventDefaultAndStop, div as _ssrg_html_div, type MouseEvent as MouseEvent, type EventAction as EventAction, type PointerEvent as PointerEvent, type ScrollEvent as ScrollEvent, type Html as Html } from "@seseragi/runtime/html"
const $ssrg$source191 = { file: "main.ssrg", line: 10, column: 3, uri: "seseragi://artifact/web-pointer-events", start: 191, end: 245 };
const $ssrg$source220 = { file: "main.ssrg", line: 10, column: 32, uri: "seseragi://artifact/web-pointer-events", start: 220, end: 244 };
const $ssrg$source321 = { file: "main.ssrg", line: 13, column: 3, uri: "seseragi://artifact/web-pointer-events", start: 321, end: 381 };
const $ssrg$source351 = { file: "main.ssrg", line: 13, column: 33, uri: "seseragi://artifact/web-pointer-events", start: 351, end: 380 };
const $ssrg$source455 = { file: "main.ssrg", line: 16, column: 3, uri: "seseragi://artifact/web-pointer-events", start: 455, end: 498 };
const $ssrg$source537 = { file: "main.ssrg", line: 19, column: 3, uri: "seseragi://artifact/web-pointer-events", start: 537, end: 883 };
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
const mouseAction = (event: MouseEvent) => $ssrg$atSource($ssrg$source191, () => _ssrg_html_DispatchPreventDefault($ssrg$atSource($ssrg$source220, () => MouseButton((event)["button"]), false)), false)
const pointerAction = (event: PointerEvent) => $ssrg$atSource($ssrg$source321, () => _ssrg_html_DispatchStopPropagation($ssrg$atSource($ssrg$source351, () => PointerKind((event)["pointerType"]), false)), false)
const scrollAction = (event: ScrollEvent) => $ssrg$atSource($ssrg$source455, () => _ssrg_html_DispatchPreventDefaultAndStop(Scrolled), false)
export const view = (_unit: undefined) => $ssrg$atSource($ssrg$source537, () => _ssrg_html_div((<$ssrg_record_type,>($ssrg_record: $ssrg_record_type) => $ssrg_record as $ssrg_record_type & { readonly [K in keyof globalThis.Object as K extends keyof $ssrg_record_type ? never : K]?: never })(({ "onClick": Clicked, "preventClickDefault": true, "stopClickPropagation": true, "onMouseDown": mouseAction, "onMouseUp": mouseAction, "onPointerDown": pointerAction, "onPointerUp": pointerAction, "onDoubleClick": mouseAction, "onContextMenu": mouseAction, "onScroll": scrollAction, "children": "Pointer target" } as const))), false)
