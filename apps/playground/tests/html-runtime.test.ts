import { describe, expect, test } from "bun:test"
import { createDomEventBindings } from "../src/runtime/browser-dom"
import {
  button,
  div,
  domEventPreventsDefault,
  form,
  input,
  label,
  messageFromDomEvent,
  renderForDom,
  renderToString,
  style,
  textarea,
  type ChangeEvent,
  type InputEvent,
} from "../../../runtime/ts/src/html"

describe("HTML browser runtime", () => {
  test("serializes record styles and CSS variables with escaping", () => {
    const node = div({
      style: style({
        variables: { cardShadow: '0 4px 16px "#0002"' },
        backgroundColor: "#fff",
        boxShadow: "var(--card-shadow)",
      }),
      children: "Styled",
    })

    expect(renderToString(node)).toBe(
      '<div style="--card-shadow: 0 4px 16px &quot;#0002&quot;; background-color: #fff; box-shadow: var(--card-shadow)">Styled</div>'
    )
  })

  test("keeps click messages out of SSR and exposes them to the DOM adapter", () => {
    const message = { tag: "Increment" } as const
    const node = button({ onClick: message, children: "+1" })

    expect(renderToString(node)).toBe('<button type="button">+1</button>')
    const rendered = renderForDom(node)
    expect(rendered.html).toBe(
      '<button data-ssrg-event-click="0" type="button">+1</button>'
    )
    expect(rendered.eventHandlers.get("0")).toEqual({
      kind: "click",
      message,
    })
  })

  test("serializes form props without leaking event handlers into SSR", () => {
    const node = form({
      onSubmit: { tag: "Submitted" },
      children: [
        label({ htmlFor: "title", children: "Title" }),
        input({
          id: "title",
          name: "title",
          value: 'Hello "Seseragi"',
          placeholder: "Type here",
          inputType: "text",
          required: true,
          onInput: (event: InputEvent) => ({
            tag: "Changed",
            value: event.value,
          }),
        }),
        textarea({
          name: "notes",
          value: "One <two>",
          disabled: true,
          onChange: (event: ChangeEvent) => ({
            tag: "Notes",
            value: event.value,
          }),
        }),
      ],
    })

    expect(renderToString(node)).toBe(
      '<form><label for="title">Title</label><input id="title" value="Hello &quot;Seseragi&quot;" name="title" required placeholder="Type here" type="text"><textarea name="notes" disabled>One &lt;two&gt;</textarea></form>'
    )
    const rendered = renderForDom(node)
    expect(rendered.html).toContain('data-ssrg-event-submit="0"')
    expect(rendered.html).toContain('data-ssrg-event-input="1"')
    expect(rendered.html).toContain('data-ssrg-event-change="2"')
    expect(renderToString(node)).not.toContain("data-ssrg-event")
  })

  test("snapshots input and change state exactly once", () => {
    type SnapshotMessage =
      | Readonly<{ tag: "Input"; snapshot: InputEvent }>
      | Readonly<{ tag: "Change"; snapshot: ChangeEvent }>
    let valueReads = 0
    let checkedReads = 0
    const target = {
      get value() {
        valueReads += 1
        return "current"
      },
      get checked() {
        checkedReads += 1
        return true
      },
    }
    const rendered = renderForDom(
      input<SnapshotMessage>({
        onInput: (event: InputEvent) => ({ tag: "Input", snapshot: event }),
        onChange: (event: ChangeEvent) => ({
          tag: "Change",
          snapshot: event,
        }),
      })
    )
    const inputHandler = rendered.eventHandlers.get("0")
    const changeHandler = rendered.eventHandlers.get("1")
    expect(inputHandler).toBeDefined()
    expect(changeHandler).toBeDefined()

    const inputMessage = messageFromDomEvent(inputHandler!, target)
    const changeMessage = messageFromDomEvent(changeHandler!, target)
    expect(inputMessage).toEqual({
      tag: "Input",
      snapshot: { value: "current" },
    })
    expect(changeMessage).toEqual({
      tag: "Change",
      snapshot: { value: "current", checked: true },
    })
    expect(Object.isFrozen(inputMessage.snapshot)).toBe(true)
    expect(Object.isFrozen(changeMessage.snapshot)).toBe(true)
    expect(valueReads).toBe(2)
    expect(checkedReads).toBe(1)
  })

  test("marks submit handlers for synchronous default prevention", () => {
    const rendered = renderForDom(
      form({ onSubmit: "Submit", children: "Send" })
    )
    const handler = rendered.eventHandlers.get("0")
    expect(handler).toBeDefined()
    expect(domEventPreventsDefault(handler!)).toBe(true)
    expect(messageFromDomEvent(handler!, {})).toBe("Submit")
  })

  test("replaces event bindings without retaining a stale handler", () => {
    const bindings = createDomEventBindings<{ value: string }>()
    bindings.replace(
      renderForDom(
        input<{ value: string }>({
          onInput: (event: InputEvent) => ({ value: `old:${event.value}` }),
        })
      )
    )
    const first = bindings.handler("0")
    expect(messageFromDomEvent(first!, { value: "draft" })).toEqual({
      value: "old:draft",
    })

    bindings.replace(
      renderForDom(
        input<{ value: string }>({
          onInput: (event: InputEvent) => ({ value: `new:${event.value}` }),
        })
      )
    )
    const current = bindings.handler("0")
    expect(messageFromDomEvent(current!, { value: "draft" })).toEqual({
      value: "new:draft",
    })
    expect(current).not.toBe(first)
  })
})
