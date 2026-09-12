import { expect, test } from "bun:test"
import {
  Dispatch,
  DispatchPreventDefault,
  type PointerEvent,
  renderForDom,
  renderToString,
  resolveDomEvent,
  type WheelEvent,
} from "../../../runtime/ts/src/html"
import { g, path, rect, svg, text, toHtml } from "../../../runtime/ts/src/svg"

test("keeps SVG namespaced construction behind an explicit bridge", () => {
  const scene = svg({
    id: "scene",
    viewBox: "0 0 400 200",
    width: 400,
    height: 200,
    children: [
      g({
        transform: "scale(1.25)",
        children: [
          path({
            d: "M 20 20 L 120 80",
            fill: "none",
            stroke: "transparent",
            strokeWidth: "16",
          }),
          rect({ x: 20, y: 30, width: 80, height: 40, rx: 8 }),
          text({ x: 60, y: 50, textAnchor: "middle", children: "node" }),
        ],
      }),
    ],
  })

  expect(renderToString(toHtml(scene))).toBe(
    '<svg id="scene" viewBox="0 0 400 200" width="400" height="200"><g transform="scale(1.25)"><path d="M 20 20 L 120 80" fill="none" stroke="transparent" stroke-width="16"></path><rect x="20" y="30" rx="8" width="80" height="40"></rect><text text-anchor="middle" x="60" y="50">node</text></g></svg>'
  )
})

test("snapshots move, cancel, and wheel events without retaining host events", () => {
  type Action =
    | { readonly kind: "move"; readonly event: PointerEvent }
    | { readonly kind: "cancel"; readonly event: PointerEvent }
    | { readonly kind: "wheel"; readonly event: WheelEvent }
  const scene = toHtml(
    svg<Action>({
      onPointerMove: (event) => Dispatch({ kind: "move", event }),
      onPointerCancel: (event) => Dispatch({ kind: "cancel", event }),
      onWheel: (event: WheelEvent) =>
        DispatchPreventDefault({ kind: "wheel", event }),
      children: [],
    })
  )
  const render = renderForDom(scene)
  const handlers = [...render.eventHandlers.values()]
  expect(render.html).toContain("data-ssrg-event-pointermove")
  expect(render.html).toContain("data-ssrg-event-pointercancel")
  expect(render.html).toContain("data-ssrg-event-wheel")

  const move = handlers.find((handler) => handler.kind === "pointermove")
  const cancel = handlers.find((handler) => handler.kind === "pointercancel")
  const wheel = handlers.find((handler) => handler.kind === "wheel")
  if (move === undefined || cancel === undefined || wheel === undefined) {
    throw new Error("missing scene event handlers")
  }
  expect(
    resolveDomEvent(
      move,
      {},
      {
        pointerId: 7,
        pointerType: "pen",
        isPrimary: true,
        button: 0,
        clientX: 12,
        clientY: 34,
        pressure: 0.75,
        altKey: false,
        ctrlKey: true,
        metaKey: false,
        shiftKey: true,
      }
    )
  ).toMatchObject({
    kind: "dispatch",
    action: {
      kind: "move",
      event: {
        pointerId: 7,
        pointerType: "pen",
        clientX: 12,
        clientY: 34,
        pressure: 0.75,
        controlKey: true,
      },
    },
  })
  expect(
    resolveDomEvent(
      cancel,
      {},
      {
        pointerId: 7,
        pointerType: "touch",
        isPrimary: true,
        button: 0,
        clientX: 0,
        clientY: 0,
        pressure: 0,
        altKey: false,
        ctrlKey: false,
        metaKey: false,
        shiftKey: false,
      }
    )
  ).toMatchObject({
    action: { kind: "cancel", event: { pointerId: 7 } },
  })
  expect(
    resolveDomEvent(
      wheel,
      {},
      {
        deltaX: 1,
        deltaY: -25,
        deltaZ: 0,
        deltaMode: 0,
        clientX: 10,
        clientY: 20,
        altKey: false,
        ctrlKey: true,
        metaKey: false,
        shiftKey: false,
      }
    )
  ).toEqual({
    kind: "dispatch",
    action: {
      kind: "wheel",
      event: {
        deltaX: 1,
        deltaY: -25,
        deltaZ: 0,
        deltaMode: 0,
        clientX: 10,
        clientY: 20,
        altKey: false,
        controlKey: true,
        metaKey: false,
        shiftKey: false,
      },
    },
    preventDefault: true,
    stopPropagation: false,
  })
})
