import { describe, expect, test } from "bun:test"
import {
  a,
  article,
  aside,
  attribute,
  audio,
  blockquote,
  body,
  br,
  button,
  type ChangeEvent,
  caption,
  code,
  custom,
  customTag,
  Dispatch,
  DispatchPreventDefault,
  DispatchPreventDefaultAndStop,
  DispatchStopPropagation,
  details,
  dialog,
  div,
  domEventPreventsDefault,
  em,
  fieldset,
  footer,
  form,
  h1,
  h2,
  h3,
  h4,
  h5,
  h6,
  head,
  header,
  hr,
  html as htmlTag,
  IgnoreEvent,
  type InputEvent,
  img,
  input,
  type KeyboardEvent,
  label,
  legend,
  li,
  link,
  type MouseEvent,
  messageFromDomEvent,
  meta,
  nav,
  ol,
  option,
  type PointerEvent,
  parseWebUrl,
  picture,
  pre,
  renderDocument,
  renderForDom,
  renderToString,
  resolveDomEvent,
  type ScrollEvent,
  select,
  small,
  source,
  strong,
  style,
  summary,
  table,
  tbody,
  td,
  textarea,
  tfoot,
  th,
  thead,
  title,
  tr,
  ul,
  video,
  type WebUrl,
} from "../../../runtime/ts/src/html"
import {
  applyDomEventResolution,
  BROWSER_DOM_EVENT_BINDINGS,
  createDomEventBindings,
} from "../../../runtime/ts/src/browser/dom"
import { createImeInputCoordinator } from "../../../runtime/ts/src/browser/ime-input"

function webUrl(value: string): WebUrl {
  const parsed = parseWebUrl(value)
  if (parsed.tag !== "Right") {
    throw new Error(`expected a valid WebUrl: ${value}`)
  }
  return parsed.value
}

describe("HTML browser runtime", () => {
  test("renders the canonical class prop for SSR and DOM", () => {
    const node = div({ class: "card featured", children: "Styled" })

    expect(renderToString(node)).toBe('<div class="card featured">Styled</div>')
    expect(renderForDom(node).html).toBe(
      '<div class="card featured">Styled</div>'
    )
  })

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

  test("renders global, ARIA, data, and validated custom attributes", () => {
    const tag = customTag("user-card")
    const ariaLabel = attribute("aria-label", 'Read "the docs"')
    const userId = attribute("data-user-id", "42")
    expect(tag.tag).toBe("Right")
    expect(ariaLabel.tag).toBe("Right")
    expect(userId.tag).toBe("Right")
    if (
      tag.tag !== "Right" ||
      ariaLabel.tag !== "Right" ||
      userId.tag !== "Right"
    ) {
      throw new Error("expected validated custom HTML values")
    }

    const node = article({
      role: "article",
      tabIndex: 0,
      lang: "en",
      dir: "ltr",
      draggable: false,
      contentEditable: true,
      attributes: [ariaLabel.value],
      children: custom(tag.value, {
        id: "mio",
        attributes: [userId.value],
        children: "Mio",
      }),
    })

    expect(renderToString(node)).toBe(
      '<article role="article" tabindex="0" lang="en" dir="ltr" draggable="false" contenteditable="true" aria-label="Read &quot;the docs&quot;"><user-card id="mio" data-user-id="42">Mio</user-card></article>'
    )
  })

  test("rejects invalid, reserved, or colliding custom names", () => {
    expect(customTag("UserCard")).toEqual({
      tag: "Left",
      value: { tag: "InvalidTagName", value: "UserCard" },
    })
    expect(attribute("bad name", "value")).toEqual({
      tag: "Left",
      value: { tag: "InvalidAttributeName", value: "bad name" },
    })
    expect(attribute("aria-Label", "value")).toEqual({
      tag: "Left",
      value: { tag: "InvalidAttributeName", value: "aria-Label" },
    })
    expect(attribute("onclick", "alert(1)")).toEqual({
      tag: "Left",
      value: { tag: "ReservedAttributeName", value: "onclick" },
    })
    expect(attribute("CLASS", "wide")).toEqual({
      tag: "Left",
      value: { tag: "ReservedAttributeName", value: "CLASS" },
    })
    expect(attribute("data-ssrg-event-click", "0")).toEqual({
      tag: "Left",
      value: {
        tag: "ReservedAttributeName",
        value: "data-ssrg-event-click",
      },
    })
  })

  test("validates WebUrl schemes, credentials, and control characters", () => {
    for (const value of [
      "",
      "/docs/getting-started?lang=ja#install",
      "guides/first-app",
      "?view=compact",
      "#reference",
      "//cdn.example.com/image.png",
      "http://example.com",
      "https://example.com/docs",
      "mailto:team@example.com",
      "tel:+81-3-1234-5678",
    ]) {
      expect(parseWebUrl(value)).toMatchObject({
        tag: "Right",
        value: { value },
      })
    }

    for (const value of [
      "javascript:alert(1)",
      "data:text/html,unsafe",
      "file:///etc/passwd",
      "ftp://example.com/file",
      "https://user@example.com/private",
      "//user:secret@example.com/private",
      "https://",
      "java\nscript:alert(1)",
    ]) {
      expect(parseWebUrl(value)).toEqual({
        tag: "Left",
        value: { tag: "UnsafeWebUrlScheme", value },
      })
    }
  })

  test("requires the opaque WebUrl value at the rendering boundary", () => {
    expect(() =>
      renderToString(a({ href: "https://example.com", children: "unsafe" }))
    ).toThrow("must be created with html.parseWebUrl")
  })

  test("renders document, sectioning, text, list, and void tags", () => {
    const document = htmlTag({
      children: [
        head({
          children: [
            title({ children: "Seseragi" }),
            meta({}),
            link({ rel: "stylesheet", href: webUrl("/styles.css") }),
          ],
        }),
        body({
          children: [
            header({ children: h1({ children: "Reference" }) }),
            nav({ children: small({ children: "Contents" }) }),
            article({
              children: [
                h2({ children: "Document" }),
                h3({ children: "Section" }),
                h4({ children: "Topic" }),
                h5({ children: "Detail" }),
                h6({ children: "Note" }),
                strong({ children: "Strong" }),
                em({ children: "Emphasis" }),
                code({ children: "let value = 1" }),
                pre({ children: "line 1\nline 2" }),
                blockquote({ children: "Typed HTML" }),
                ul({ children: [li({ children: "One" })] }),
                ol({ children: [li({ children: "First" })] }),
                br({}),
                hr({}),
              ],
            }),
            aside({ children: "Related" }),
            footer({ children: "End" }),
          ],
        }),
      ],
    })

    expect(renderDocument(document)).toBe(
      [
        '<!doctype html><html><head><title>Seseragi</title><meta><link rel="stylesheet" href="/styles.css">',
        "</head><body><header><h1>Reference</h1></header>",
        "<nav><small>Contents</small></nav><article>",
        "<h2>Document</h2><h3>Section</h3><h4>Topic</h4>",
        "<h5>Detail</h5><h6>Note</h6><strong>Strong</strong>",
        "<em>Emphasis</em><code>let value = 1</code>",
        "<pre>line 1\nline 2</pre><blockquote>Typed HTML</blockquote>",
        "<ul><li>One</li></ul><ol><li>First</li></ol><br><hr>",
        "</article><aside>Related</aside><footer>End</footer>",
        "</body></html>",
      ].join("")
    )
  })

  test("renders link, image, picture, source, video, and audio props", () => {
    const node = article({
      children: [
        a({
          href: webUrl("https://example.com/docs?q=seseragi"),
          target: "_blank",
          rel: "noopener",
          download: true,
          children: "Docs",
        }),
        img({
          src: webUrl("/assets/hero.png"),
          alt: 'Seseragi "hero"',
          width: 640,
          height: 360,
          loading: "lazy",
        }),
        picture({
          children: source({
            src: webUrl("/assets/hero-wide.png"),
            media: "(min-width: 48rem)",
            mimeType: "image/png",
          }),
        }),
        video({
          src: webUrl("/assets/intro.mp4"),
          width: 640,
          height: 360,
          children: source({
            src: webUrl("/assets/intro.webm"),
            mimeType: "video/webm",
          }),
        }),
        audio({
          src: webUrl("/assets/theme.mp3"),
          children: source({
            src: webUrl("/assets/theme.ogg"),
            mimeType: "audio/ogg",
          }),
        }),
      ],
    })

    expect(renderToString(node)).toBe(
      [
        '<article><a href="https://example.com/docs?q=seseragi" target="_blank" rel="noopener" download>Docs</a>',
        '<img src="/assets/hero.png" alt="Seseragi &quot;hero&quot;" width="640" height="360" loading="lazy">',
        '<picture><source src="/assets/hero-wide.png" media="(min-width: 48rem)" type="image/png"></picture>',
        '<video src="/assets/intro.mp4" width="640" height="360"><source src="/assets/intro.webm" type="video/webm"></video>',
        '<audio src="/assets/theme.mp3"><source src="/assets/theme.ogg" type="audio/ogg"></audio></article>',
      ].join("")
    )
  })

  test("rejects children passed to void elements at the runtime boundary", () => {
    expect(() =>
      img({ src: webUrl("/hero.png"), alt: "Hero", children: "invalid" })
    ).toThrow("void HTML element img cannot have children")
    expect(() =>
      source({ src: webUrl("/hero.png"), children: "invalid" })
    ).toThrow("void HTML element source cannot have children")
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
      preventDefault: false,
      stopPropagation: false,
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

  test("renders form, table, and interactive tag-specific props", () => {
    const node = div({
      children: [
        form({
          name: "profile",
          autoComplete: "on",
          children: fieldset({
            children: [
              legend({ children: "Profile" }),
              label({ htmlFor: "age", children: "Age" }),
              input({
                id: "age",
                name: "age",
                value: "18",
                readOnly: true,
                multiple: true,
                autoComplete: "off",
                autoFocus: true,
                min: "0",
                max: "120",
                step: "1",
                pattern: "[0-9]+",
              }),
              textarea({
                name: "bio",
                value: "Typed UI",
                readOnly: true,
                autoComplete: "off",
                autoFocus: true,
                rows: 4,
                cols: 40,
              }),
              select({
                name: "theme",
                value: "dark",
                required: true,
                multiple: true,
                autoFocus: true,
                onChange: (event: ChangeEvent) => event.value,
                children: [
                  option({ value: "light", disabled: true, children: "Light" }),
                  option({ value: "dark", selected: true, children: "Dark" }),
                ],
              }),
            ],
          }),
        }),
        table({
          children: [
            caption({ children: "Scores" }),
            thead({
              children: tr({
                children: th({ colSpan: 2, children: "Result" }),
              }),
            }),
            tbody({
              children: tr({
                children: td({ rowSpan: 2, children: "42" }),
              }),
            }),
            tfoot({ children: tr({ children: td({ children: "End" }) }) }),
          ],
        }),
        details({
          open: true,
          children: summary({ children: "More details" }),
        }),
        dialog({ open: true, children: "Ready" }),
      ],
    })

    expect(renderToString(node)).toBe(
      [
        '<div><form name="profile" autocomplete="on"><fieldset><legend>Profile</legend><label for="age">Age</label>',
        '<input id="age" value="18" name="age" readonly multiple autocomplete="off" autofocus min="0" max="120" step="1" pattern="[0-9]+" type="text">',
        '<textarea name="bio" readonly autocomplete="off" autofocus rows="4" cols="40">Typed UI</textarea>',
        '<select name="theme" value="dark" required multiple autofocus><option value="light" disabled>Light</option>',
        '<option value="dark" selected>Dark</option></select></fieldset></form>',
        '<table><caption>Scores</caption><thead><tr><th colspan="2">Result</th></tr></thead>',
        '<tbody><tr><td rowspan="2">42</td></tr></tbody><tfoot><tr><td>End</td></tr></tfoot></table>',
        "<details open><summary>More details</summary></details><dialog open>Ready</dialog></div>",
      ].join("")
    )
  })

  test("integrates accessible document, form, table, and controlled events", () => {
    type IntegratedAction =
      | Readonly<{ tag: "Submitted" }>
      | Readonly<{ tag: "Changed"; value: string }>
      | Readonly<{ tag: "Pointer"; pointerType: string }>
      | Readonly<{ tag: "Filtered" }>
      | Readonly<{ tag: "Deleted" }>
    const ariaLive = attribute("aria-live", "polite")
    expect(ariaLive.tag).toBe("Right")
    if (ariaLive.tag !== "Right") {
      throw new Error("expected a validated ARIA attribute")
    }

    const node = article<IntegratedAction>({
      role: "region",
      attributes: [ariaLive.value],
      children: [
        a({
          href: webUrl("https://example.com/docs"),
          children: img({
            src: webUrl("/seseragi-mark.png"),
            alt: "Seseragi documentation",
            width: 72,
            height: 72,
          }),
        }),
        form({
          onSubmit: { tag: "Submitted" },
          children: [
            label({ htmlFor: "task-title", children: "Task" }),
            input({
              id: "task-title",
              name: "title",
              required: true,
              onInput: (event: InputEvent) => ({
                tag: "Changed",
                value: event.value,
              }),
            }),
            button({ buttonType: "submit", children: "Add task" }),
          ],
        }),
        table({
          children: [
            caption({ children: "Todo status" }),
            thead({
              children: tr({
                children: [
                  th({ children: "Task" }),
                  th({ children: "Action" }),
                ],
              }),
            }),
            tbody({
              children: tr({
                onPointerDown: (event: PointerEvent) =>
                  Dispatch({
                    tag: "Pointer",
                    pointerType: event.pointerType,
                  }),
                children: [
                  td({ children: "Ship integration" }),
                  td({
                    children: button({
                      onClick: { tag: "Deleted" },
                      preventClickDefault: true,
                      stopClickPropagation: true,
                      children: "Delete",
                    }),
                  }),
                ],
              }),
            }),
          ],
        }),
        button({
          onKeyDown: (event: KeyboardEvent) =>
            event.key === "ArrowRight"
              ? DispatchPreventDefault({ tag: "Filtered" })
              : IgnoreEvent,
          children: "Urgent filter",
        }),
      ],
    })

    expect(renderToString(node)).toBe(
      [
        '<article role="region" aria-live="polite">',
        '<a href="https://example.com/docs"><img src="/seseragi-mark.png" alt="Seseragi documentation" width="72" height="72"></a>',
        '<form><label for="task-title">Task</label><input id="task-title" name="title" required type="text">',
        '<button type="submit">Add task</button></form>',
        "<table><caption>Todo status</caption><thead><tr><th>Task</th><th>Action</th></tr></thead>",
        '<tbody><tr><td>Ship integration</td><td><button type="button">Delete</button></td></tr></tbody></table>',
        '<button type="button">Urgent filter</button></article>',
      ].join("")
    )

    const rendered = renderForDom(node)
    const handlers = [...rendered.eventHandlers.values()]
    const inputHandler = handlers.find((handler) => handler.kind === "input")
    const submitHandler = handlers.find((handler) => handler.kind === "submit")
    const pointerHandler = handlers.find(
      (handler) => handler.kind === "pointerdown"
    )
    const clickHandler = handlers.find((handler) => handler.kind === "click")
    const keyHandler = handlers.find((handler) => handler.kind === "keydown")
    expect(
      messageFromDomEvent<IntegratedAction>(inputHandler!, {
        value: "Edited on iOS",
      })
    ).toEqual({
      tag: "Changed",
      value: "Edited on iOS",
    })
    expect(domEventPreventsDefault(submitHandler!)).toBe(true)
    expect(messageFromDomEvent<IntegratedAction>(submitHandler!, {})).toEqual({
      tag: "Submitted",
    })
    expect(
      resolveDomEvent<IntegratedAction>(
        pointerHandler!,
        {},
        {
          pointerId: 9,
          pointerType: "touch",
          isPrimary: true,
          button: 0,
          clientX: 12,
          clientY: 24,
          pressure: 0.5,
          altKey: false,
          ctrlKey: false,
          metaKey: false,
          shiftKey: false,
        }
      )
    ).toMatchObject({
      kind: "dispatch",
      action: { tag: "Pointer", pointerType: "touch" },
    })
    expect(
      resolveDomEvent<IntegratedAction>(
        keyHandler!,
        {},
        {
          key: "ArrowRight",
          code: "ArrowRight",
          repeat: false,
          altKey: false,
          ctrlKey: false,
          metaKey: false,
          shiftKey: false,
        }
      )
    ).toEqual({
      kind: "dispatch",
      action: { tag: "Filtered" },
      preventDefault: true,
      stopPropagation: false,
    })

    const order: string[] = []
    applyDomEventResolution(
      {
        preventDefault: () => order.push("preventDefault"),
        stopPropagation: () => order.push("stopPropagation"),
      },
      resolveDomEvent<IntegratedAction>(clickHandler!, {}),
      (action) => order.push(`dispatch:${action.tag}`)
    )
    expect(order).toEqual([
      "preventDefault",
      "stopPropagation",
      "dispatch:Deleted",
    ])
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

  test("dispatches focus and immutable keyboard snapshots", () => {
    type Action = Readonly<{
      readonly tag: string
      readonly keyboard?: KeyboardEvent
    }>
    const rendered = renderForDom(
      button<Action>({
        onFocus: { tag: "Focused" },
        onBlur: { tag: "Blurred" },
        onKeyDown: (keyboard: KeyboardEvent) =>
          Dispatch({ tag: "KeyDown", keyboard }),
        onKeyUp: (keyboard: KeyboardEvent) =>
          Dispatch({ tag: "KeyUp", keyboard }),
        children: "Keyboard target",
      })
    )

    expect(rendered.html).toContain('data-ssrg-event-focus="0"')
    expect(rendered.html).toContain('data-ssrg-event-blur="1"')
    expect(rendered.html).toContain('data-ssrg-event-keydown="2"')
    expect(rendered.html).toContain('data-ssrg-event-keyup="3"')
    expect(messageFromDomEvent(rendered.eventHandlers.get("0")!, {})).toEqual({
      tag: "Focused",
    })
    expect(messageFromDomEvent(rendered.eventHandlers.get("1")!, {})).toEqual({
      tag: "Blurred",
    })

    let reads = 0
    const nativeEvent = {
      get key() {
        reads += 1
        return "Enter"
      },
      get code() {
        reads += 1
        return "Enter"
      },
      get repeat() {
        reads += 1
        return false
      },
      get altKey() {
        reads += 1
        return false
      },
      get ctrlKey() {
        reads += 1
        return true
      },
      get metaKey() {
        reads += 1
        return false
      },
      get shiftKey() {
        reads += 1
        return true
      },
    }
    const action = messageFromDomEvent(
      rendered.eventHandlers.get("2")!,
      {},
      nativeEvent
    )
    expect(action).toEqual({
      tag: "KeyDown",
      keyboard: {
        key: "Enter",
        code: "Enter",
        repeat: false,
        altKey: false,
        controlKey: true,
        metaKey: false,
        shiftKey: true,
      },
    })
    expect(Object.isFrozen(action.keyboard)).toBe(true)
    expect(reads).toBe(7)
    expect(renderToString(button({ onFocus: "Focus", children: "SSR" }))).toBe(
      '<button type="button">SSR</button>'
    )
  })

  test("normalizes bubbling focus and keyboard browser events", () => {
    expect(BROWSER_DOM_EVENT_BINDINGS).toContainEqual({
      nativeKind: "focusin",
      handlerKind: "focus",
      capture: false,
    })
    expect(BROWSER_DOM_EVENT_BINDINGS).toContainEqual({
      nativeKind: "focusout",
      handlerKind: "blur",
      capture: false,
    })
    expect(BROWSER_DOM_EVENT_BINDINGS).toContainEqual({
      nativeKind: "keydown",
      handlerKind: "keydown",
      capture: false,
    })
    expect(BROWSER_DOM_EVENT_BINDINGS).toContainEqual({
      nativeKind: "keyup",
      handlerKind: "keyup",
      capture: false,
    })
  })

  test("snapshots mouse, pointer, and scroll events without host references", () => {
    type Action = Readonly<{
      readonly tag: string
      readonly snapshot: MouseEvent | PointerEvent | ScrollEvent
    }>
    const rendered = renderForDom(
      div<Action>({
        onMouseDown: (snapshot: MouseEvent) =>
          DispatchPreventDefault({ tag: "Mouse", snapshot }),
        onPointerDown: (snapshot: PointerEvent) =>
          DispatchStopPropagation({ tag: "Pointer", snapshot }),
        onScroll: (snapshot: ScrollEvent) =>
          DispatchPreventDefaultAndStop({ tag: "Scroll", snapshot }),
        children: "Events",
      })
    )

    const mouse = resolveDomEvent(
      rendered.eventHandlers.get("0")!,
      {},
      {
        button: 2,
        clientX: 12.5,
        clientY: 24.25,
        altKey: true,
        ctrlKey: false,
        metaKey: false,
        shiftKey: true,
      }
    )
    expect(mouse).toEqual({
      kind: "dispatch",
      action: {
        tag: "Mouse",
        snapshot: {
          button: 2,
          clientX: 12.5,
          clientY: 24.25,
          altKey: true,
          controlKey: false,
          metaKey: false,
          shiftKey: true,
        },
      },
      preventDefault: true,
      stopPropagation: false,
    })
    if (mouse.kind !== "dispatch") throw new Error("expected mouse dispatch")
    expect(Object.isFrozen(mouse.action.snapshot)).toBe(true)

    for (const pointerType of ["mouse", "touch", "pen"]) {
      const pointer = resolveDomEvent(
        rendered.eventHandlers.get("1")!,
        {},
        {
          pointerId: 7,
          pointerType,
          isPrimary: true,
          button: 0,
          clientX: 3,
          clientY: 4,
          pressure: pointerType === "mouse" ? 0 : 0.5,
          altKey: false,
          ctrlKey: true,
          metaKey: false,
          shiftKey: false,
        }
      )
      expect(pointer.kind).toBe("dispatch")
      if (pointer.kind !== "dispatch") continue
      expect(pointer.action.snapshot).toMatchObject({
        pointerId: 7,
        pointerType,
        isPrimary: true,
        button: 0,
      })
      expect(Object.isFrozen(pointer.action.snapshot)).toBe(true)
      expect(pointer.stopPropagation).toBe(true)
    }

    const scroll = resolveDomEvent(rendered.eventHandlers.get("2")!, {
      scrollLeft: 8.5,
      scrollTop: 144,
    })
    expect(scroll).toMatchObject({
      kind: "dispatch",
      action: {
        tag: "Scroll",
        snapshot: { scrollLeft: 8.5, scrollTop: 144 },
      },
      preventDefault: true,
      stopPropagation: true,
    })
    expect(rendered.html).toContain('data-ssrg-event-mousedown="0"')
    expect(rendered.html).toContain('data-ssrg-event-pointerdown="1"')
    expect(rendered.html).toContain('data-ssrg-event-scroll="2"')
  })

  test("applies event controls before enqueue and can ignore an event", () => {
    const order: string[] = []
    const rendered = renderForDom(
      div<string>({
        onContextMenu: () => {
          order.push("mapper")
          return DispatchPreventDefaultAndStop("Open menu")
        },
        onDoubleClick: () => IgnoreEvent,
        children: "Controlled",
      })
    )
    const resolution = resolveDomEvent(
      rendered.eventHandlers.get("1")!,
      {},
      {
        button: 2,
        clientX: 0,
        clientY: 0,
        altKey: false,
        ctrlKey: false,
        metaKey: false,
        shiftKey: false,
      }
    )
    applyDomEventResolution(
      {
        preventDefault: () => order.push("preventDefault"),
        stopPropagation: () => order.push("stopPropagation"),
      },
      resolution,
      (action) => order.push(`enqueue:${action}`)
    )
    expect(order).toEqual([
      "mapper",
      "preventDefault",
      "stopPropagation",
      "enqueue:Open menu",
    ])

    const ignored = resolveDomEvent(
      rendered.eventHandlers.get("0")!,
      {},
      {
        button: 0,
        clientX: 0,
        clientY: 0,
        altKey: false,
        ctrlKey: false,
        metaKey: false,
        shiftKey: false,
      }
    )
    applyDomEventResolution(
      {
        preventDefault: () => order.push("unexpected preventDefault"),
        stopPropagation: () => order.push("unexpected stopPropagation"),
      },
      ignored,
      () => order.push("unexpected enqueue")
    )
    expect(ignored.kind).toBe("ignore")
    expect(order).toHaveLength(4)
  })

  test("captures scroll events and preserves click control compatibility", () => {
    expect(BROWSER_DOM_EVENT_BINDINGS).toContainEqual({
      nativeKind: "scroll",
      handlerKind: "scroll",
      capture: true,
    })
    const rendered = renderForDom(
      button({
        onClick: "Navigate",
        preventClickDefault: true,
        stopClickPropagation: true,
        children: "Open",
      })
    )
    expect(resolveDomEvent(rendered.eventHandlers.get("0")!, {})).toEqual({
      kind: "dispatch",
      action: "Navigate",
      preventDefault: true,
      stopPropagation: true,
    })
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
