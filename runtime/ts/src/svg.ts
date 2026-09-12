import {
  createSvgElement,
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
