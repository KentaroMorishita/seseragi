import { describe, expect, test } from "bun:test"
import {
  button,
  type ChangeEvent,
  div,
  domEventPreventsDefault,
  form,
  type InputEvent,
  input,
  label,
  messageFromDomEvent,
  renderForDom,
  renderToString,
  style,
  textarea,
} from "../../../runtime/ts/src/html"
import { createDomEventBindings } from "../src/runtime/browser-dom"
import { createImeInputCoordinator } from "../src/runtime/ime-input"

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

  test("keeps click actions out of SSR and exposes them to the DOM adapter", () => {
    const action = { tag: "Increment" } as const
    const node = button({ onClick: action, children: "+1" })

    expect(renderToString(node)).toBe('<button type="button">+1</button>')
    const rendered = renderForDom(node)
    expect(rendered.html).toBe(
      '<button data-ssrg-event-click="0" type="button">+1</button>'
    )
    expect(rendered.eventHandlers.get("0")).toEqual({
      kind: "click",
      message: action,
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
    type SnapshotAction =
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
      input<SnapshotAction>({
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

    const inputAction = messageFromDomEvent(inputHandler!, target)
    const changeAction = messageFromDomEvent(changeHandler!, target)
    expect(inputAction).toEqual({
      tag: "Input",
      snapshot: { value: "current" },
    })
    expect(changeAction).toEqual({
      tag: "Change",
      snapshot: { value: "current", checked: true },
    })
    expect(Object.isFrozen(inputAction.snapshot)).toBe(true)
    expect(Object.isFrozen(changeAction.snapshot)).toBe(true)
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

describe("IME input coordination", () => {
  test("commits Japanese conversion values exactly once", () => {
    for (const expected of ["日本語", "ひらがな", "カタカナ", "ＡＢＣ１２３"]) {
      const ime = createImeInputCoordinator<{ value: string }>()
      const control = { value: "" }
      const actions: string[] = []

      ime.start(control)
      control.value = expected.slice(0, 1)
      expect(ime.input(control, true)).toBe(false)
      control.value = expected
      expect(ime.end(control)).toBe(true)
      expect(ime.input(control, false)).toBe(false)
      if (ime.finalize(control)) actions.push(control.value)
      expect(actions).toEqual([expected])
      expect(ime.finalize(control)).toBe(false)
    }
  })

  test("coalesces composition input before and after compositionend", () => {
    const ime = createImeInputCoordinator<object>()
    const input = {}

    ime.start(input)
    expect(ime.input(input, true)).toBe(false)
    ime.update(input)
    expect(ime.input(input, false)).toBe(false)
    expect(ime.end(input)).toBe(true)
    expect(ime.busy()).toBe(true)
    expect(ime.input(input, false)).toBe(false)
    expect(ime.finalize(input)).toBe(true)
    expect(ime.finalize(input)).toBe(false)
    expect(ime.busy()).toBe(false)
    expect(ime.input(input, false)).toBe(true)
  })

  test("uses native isComposing when compositionstart is absent", () => {
    const ime = createImeInputCoordinator<object>()
    const textarea = {}

    expect(ime.input(textarea, true)).toBe(false)
    expect(ime.input(textarea, false)).toBe(false)
    expect(ime.end(textarea)).toBe(true)
    expect(ime.finalize(textarea)).toBe(true)
    expect(ime.busy()).toBe(false)
  })

  test("commits an unfinished composition before submit", () => {
    const ime = createImeInputCoordinator<object>()
    const input = {}
    const textarea = {}

    ime.start(input)
    ime.start(textarea)
    expect(ime.targets()).toEqual([input, textarea])
    expect(ime.commit(input)).toBe(true)
    expect(ime.finalize(input)).toBe(true)
    expect(ime.commit(input)).toBe(false)
    expect(ime.targets()).toEqual([textarea])
    ime.reset()
    expect(ime.busy()).toBe(false)
  })
})
