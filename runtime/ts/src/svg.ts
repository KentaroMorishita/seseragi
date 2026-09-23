import {
  type BindingTarget,
  numberAttributeTarget,
  stringAttributeTarget,
} from "./dom"
import {
  createSvgElement,
  type ElementRef,
  type EventAction,
  type Html,
  type PointerEvent,
  type WheelEvent,
} from "./html"

const SVG_NODE = Symbol("seseragi.svg")

type PhantomAction<Action> = Readonly<{ readonly __action?: Action }>

/** Immutable SVG scene node, kept separate from the HTML element surface. */
export type Svg<Action> = PhantomAction<Action> &
  Readonly<{
    readonly [SVG_NODE]: Html<Action>
  }>

type SvgProps<Action> = Readonly<Record<string, unknown>> &
  Readonly<{
    readonly children?: ReadonlyArray<Svg<Action>> | Svg<Action> | string
    readonly onPointerDown?: (event: PointerEvent) => EventAction<Action>
    readonly onPointerMove?: (event: PointerEvent) => EventAction<Action>
    readonly onPointerUp?: (event: PointerEvent) => EventAction<Action>
    readonly onPointerCancel?: (event: PointerEvent) => EventAction<Action>
    readonly onWheel?: (event: WheelEvent) => EventAction<Action>
  }>

function unwrapChildren<Action>(value: unknown): unknown {
  if (value === undefined || typeof value === "string") return value
  if (isSvg<Action>(value)) return value[SVG_NODE]
  if (Array.isArray(value)) {
    return Object.freeze(
      value.map((child) => {
        if (!isSvg<Action>(child)) {
          throw new TypeError("SVG child arrays may contain only Svg values")
        }
        return child[SVG_NODE]
      })
    )
  }
  throw new TypeError("unsupported SVG children value")
}

function element<Action>(name: string, value: SvgProps<Action>): Svg<Action> {
  const props = { ...value, children: unwrapChildren<Action>(value.children) }
  return Object.freeze({
    [SVG_NODE]: createSvgElement<Action>(name, props),
  })
}

function tag(name: string) {
  return <Action = never>(props: SvgProps<Action>): Svg<Action> =>
    element(name, props)
}

export const svg = tag("svg")
export const g = tag("g")
export const rect = tag("rect")
export const path = tag("path")
export const circle = tag("circle")
export const line = tag("line")
export const polyline = tag("polyline")
export const polygon = tag("polygon")
export const text = tag("text")

export function viewBoxTarget<Action>(
  reference: ElementRef
): BindingTarget<Action, string> {
  return stringAttributeTarget(reference, "viewBox", {
    namespace: "svg",
    tags: ["svg"],
  })
}

export function transformTarget<Action>(
  reference: ElementRef
): BindingTarget<Action, string> {
  return stringAttributeTarget(reference, "transform", { namespace: "svg" })
}

export function pathDataTarget<Action>(
  reference: ElementRef
): BindingTarget<Action, string> {
  return stringAttributeTarget(reference, "d", {
    namespace: "svg",
    tags: ["path"],
  })
}

export function pointsTarget<Action>(
  reference: ElementRef
): BindingTarget<Action, string> {
  return stringAttributeTarget(reference, "points", {
    namespace: "svg",
    tags: ["polyline", "polygon"],
  })
}

function svgStringTarget<Action>(
  reference: ElementRef,
  name: string
): BindingTarget<Action, string> {
  return stringAttributeTarget(reference, name, { namespace: "svg" })
}

export function fillTarget<Action>(
  reference: ElementRef
): BindingTarget<Action, string> {
  return svgStringTarget(reference, "fill")
}

export function strokeTarget<Action>(
  reference: ElementRef
): BindingTarget<Action, string> {
  return svgStringTarget(reference, "stroke")
}

export function strokeWidthTarget<Action>(
  reference: ElementRef
): BindingTarget<Action, string> {
  return svgStringTarget(reference, "stroke-width")
}

export function pointerEventsTarget<Action>(
  reference: ElementRef
): BindingTarget<Action, string> {
  return svgStringTarget(reference, "pointer-events")
}

export function ariaLabelTarget<Action>(
  reference: ElementRef
): BindingTarget<Action, string> {
  return svgStringTarget(reference, "aria-label")
}

function svgNumberTarget<Action>(
  reference: ElementRef,
  name: string
): BindingTarget<Action, number> {
  return numberAttributeTarget(reference, name, { namespace: "svg" })
}

export function xTarget<Action>(
  reference: ElementRef
): BindingTarget<Action, number> {
  return svgNumberTarget(reference, "x")
}

export function yTarget<Action>(
  reference: ElementRef
): BindingTarget<Action, number> {
  return svgNumberTarget(reference, "y")
}

export function x1Target<Action>(
  reference: ElementRef
): BindingTarget<Action, number> {
  return svgNumberTarget(reference, "x1")
}

export function y1Target<Action>(
  reference: ElementRef
): BindingTarget<Action, number> {
  return svgNumberTarget(reference, "y1")
}

export function x2Target<Action>(
  reference: ElementRef
): BindingTarget<Action, number> {
  return svgNumberTarget(reference, "x2")
}

export function y2Target<Action>(
  reference: ElementRef
): BindingTarget<Action, number> {
  return svgNumberTarget(reference, "y2")
}

export function cxTarget<Action>(
  reference: ElementRef
): BindingTarget<Action, number> {
  return svgNumberTarget(reference, "cx")
}

export function cyTarget<Action>(
  reference: ElementRef
): BindingTarget<Action, number> {
  return svgNumberTarget(reference, "cy")
}

export function radiusTarget<Action>(
  reference: ElementRef
): BindingTarget<Action, number> {
  return svgNumberTarget(reference, "r")
}

export function rxTarget<Action>(
  reference: ElementRef
): BindingTarget<Action, number> {
  return svgNumberTarget(reference, "rx")
}

export function ryTarget<Action>(
  reference: ElementRef
): BindingTarget<Action, number> {
  return svgNumberTarget(reference, "ry")
}

export function widthTarget<Action>(
  reference: ElementRef
): BindingTarget<Action, number> {
  return svgNumberTarget(reference, "width")
}

export function heightTarget<Action>(
  reference: ElementRef
): BindingTarget<Action, number> {
  return svgNumberTarget(reference, "height")
}

export function opacityTarget<Action>(
  reference: ElementRef
): BindingTarget<Action, number> {
  return svgNumberTarget(reference, "opacity")
}

/** Explicit namespace bridge accepted by std/web/dom. */
export function toHtml<Action>(value: Svg<Action>): Html<Action> {
  if (!isSvg<Action>(value)) throw new TypeError("expected std/web/svg Svg")
  return value[SVG_NODE]
}

function isSvg<Action>(value: unknown): value is Svg<Action> {
  return (
    typeof value === "object" && value !== null && SVG_NODE in (value as object)
  )
}
