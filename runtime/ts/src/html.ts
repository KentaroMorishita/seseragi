import { type Either, Left, Right } from "./sum"
import { type File, wrapFile } from "./web-file"

const HTML_NODE = Symbol("seseragi.html")
const STYLE = Symbol("seseragi.style")
const TAG = Symbol("seseragi.html.tag")
const ATTRIBUTE = Symbol("seseragi.html.attribute")
const WEB_URL = Symbol("seseragi.html.web-url")

type PhantomAction<Action> = {
  readonly __action?: Action
}

type TextNode<Action> = PhantomAction<Action> &
  Readonly<{
    readonly [HTML_NODE]: "text"
    readonly value: string
  }>

type FragmentNode<Action> = PhantomAction<Action> &
  Readonly<{
    readonly [HTML_NODE]: "fragment"
    readonly children: ReadonlyArray<Html<Action>>
  }>

type ElementNode<Action> = PhantomAction<Action> &
  Readonly<{
    readonly [HTML_NODE]: "element"
    readonly tag: string
    readonly props: Readonly<Record<string, unknown>>
    readonly children: ReadonlyArray<Html<Action>>
    readonly voidElement: boolean
  }>

/** Immutable pure HTML tree. The action parameter is a compile-time phantom. */
export type Html<Action> =
  | TextNode<Action>
  | FragmentNode<Action>
  | ElementNode<Action>

/** Immutable text-input snapshot. It never exposes the host DOM event. */
export type InputEvent = Readonly<{
  readonly value: string
}>

/** Immutable change snapshot shared by text and checked controls. */
export type ChangeEvent = Readonly<{
  readonly value: string
  readonly checked: boolean
}>

/** Immutable file-input snapshot. It retains only opaque File handles. */
export type FileChangeEvent = Readonly<{
  readonly files: ReadonlyArray<File>
}>

/** Immutable keyboard snapshot. It never retains the host DOM event. */
export type KeyboardEvent = Readonly<{
  readonly key: string
  readonly code: string
  readonly repeat: boolean
  readonly altKey: boolean
  readonly controlKey: boolean
  readonly metaKey: boolean
  readonly shiftKey: boolean
}>

export type MouseEvent = Readonly<{
  readonly button: number
  readonly clientX: number
  readonly clientY: number
  readonly altKey: boolean
  readonly controlKey: boolean
  readonly metaKey: boolean
  readonly shiftKey: boolean
}>

export type PointerEvent = Readonly<{
  readonly pointerId: number
  readonly pointerType: string
  readonly isPrimary: boolean
  readonly button: number
  readonly clientX: number
  readonly clientY: number
  readonly pressure: number
  readonly altKey: boolean
  readonly controlKey: boolean
  readonly metaKey: boolean
  readonly shiftKey: boolean
}>

export type ScrollEvent = Readonly<{
  readonly scrollLeft: number
  readonly scrollTop: number
}>

export type EventAction<Action> =
  | Readonly<{ readonly tag: "IgnoreEvent" }>
  | Readonly<{ readonly tag: "Dispatch"; readonly value: Action }>
  | Readonly<{
      readonly tag: "DispatchPreventDefault"
      readonly value: Action
    }>
  | Readonly<{
      readonly tag: "DispatchStopPropagation"
      readonly value: Action
    }>
  | Readonly<{
      readonly tag: "DispatchPreventDefaultAndStop"
      readonly value: Action
    }>

export const IgnoreEvent: EventAction<never> = Object.freeze({
  tag: "IgnoreEvent",
})
export const Dispatch = <Action>(value: Action): EventAction<Action> =>
  Object.freeze({ tag: "Dispatch", value })
export const DispatchPreventDefault = <Action>(
  value: Action
): EventAction<Action> =>
  Object.freeze({ tag: "DispatchPreventDefault", value })
export const DispatchStopPropagation = <Action>(
  value: Action
): EventAction<Action> =>
  Object.freeze({ tag: "DispatchStopPropagation", value })
export const DispatchPreventDefaultAndStop = <Action>(
  value: Action
): EventAction<Action> =>
  Object.freeze({ tag: "DispatchPreventDefaultAndStop", value })

export type HtmlBuildError =
  | Readonly<{ readonly tag: "InvalidTagName"; readonly value: string }>
  | Readonly<{ readonly tag: "InvalidAttributeName"; readonly value: string }>
  | Readonly<{ readonly tag: "ReservedAttributeName"; readonly value: string }>
  | Readonly<{ readonly tag: "UnsafeWebUrlScheme"; readonly value: string }>

export const InvalidTagName = (value: string): HtmlBuildError =>
  Object.freeze({ tag: "InvalidTagName", value })
export const InvalidAttributeName = (value: string): HtmlBuildError =>
  Object.freeze({ tag: "InvalidAttributeName", value })
export const ReservedAttributeName = (value: string): HtmlBuildError =>
  Object.freeze({ tag: "ReservedAttributeName", value })
export const UnsafeWebUrlScheme = (value: string): HtmlBuildError =>
  Object.freeze({ tag: "UnsafeWebUrlScheme", value })

export type Tag = Readonly<{
  readonly [TAG]: true
  readonly name: string
}>

export type Attribute = Readonly<{
  readonly [ATTRIBUTE]: true
  readonly name: string
  readonly value: string
}>

/** Opaque URL accepted by the standard Web UI security contract. */
export type WebUrl = Readonly<{
  readonly [WEB_URL]: true
  readonly value: string
}>

export type DomEventHandler<Action> =
  | Readonly<{
      readonly kind: "click"
      readonly message: Action
      readonly preventDefault: boolean
      readonly stopPropagation: boolean
    }>
  | Readonly<{ readonly kind: "focus"; readonly message: Action }>
  | Readonly<{ readonly kind: "blur"; readonly message: Action }>
  | Readonly<{
      readonly kind: "keydown"
      readonly map: (event: KeyboardEvent) => EventAction<Action>
    }>
  | Readonly<{
      readonly kind: "keyup"
      readonly map: (event: KeyboardEvent) => EventAction<Action>
    }>
  | Readonly<{
      readonly kind: "mousedown" | "mouseup" | "dblclick" | "contextmenu"
      readonly map: (event: MouseEvent) => EventAction<Action>
    }>
  | Readonly<{
      readonly kind: "pointerdown" | "pointerup"
      readonly map: (event: PointerEvent) => EventAction<Action>
    }>
  | Readonly<{
      readonly kind: "scroll"
      readonly map: (event: ScrollEvent) => EventAction<Action>
    }>
  | Readonly<{
      readonly kind: "input"
      readonly map: (event: InputEvent) => Action
    }>
  | Readonly<{
      readonly kind: "change"
      readonly map: (event: ChangeEvent) => Action
    }>
  | Readonly<{
      readonly kind: "file-change"
      readonly map: (event: FileChangeEvent) => Action
    }>
  | Readonly<{ readonly kind: "submit"; readonly message: Action }>

export type DomEventResolution<Action> =
  | Readonly<{
      readonly kind: "ignore"
      readonly preventDefault: false
      readonly stopPropagation: false
    }>
  | Readonly<{
      readonly kind: "dispatch"
      readonly action: Action
      readonly preventDefault: boolean
      readonly stopPropagation: boolean
    }>

export type DomRender<Action> = Readonly<{
  readonly html: string
  readonly eventHandlers: ReadonlyMap<string, DomEventHandler<Action>>
}>

/** Immutable serialized inline style created from a checked Seseragi record. */
export type Style = Readonly<{
  readonly [STYLE]: true
  readonly cssText: string
}>

export function style(declarations: unknown): Style {
  const record = expectRecord(declarations, "HTML style declarations")
  const properties: string[] = []
  for (const [name, value] of Object.entries(record)) {
    if (name === "variables") {
      const variables = expectRecord(value, "HTML style variables")
      for (const [variable, variableValue] of Object.entries(variables)) {
        properties.push(
          `--${camelToKebab(variable)}: ${expectStyleValue(variable, variableValue)}`
        )
      }
      continue
    }
    properties.push(`${camelToKebab(name)}: ${expectStyleValue(name, value)}`)
  }
  return Object.freeze({
    [STYLE]: true as const,
    cssText: properties.join("; "),
  })
}

export function customTag(name: string): Either<HtmlBuildError, Tag> {
  if (!/^[a-z][a-z0-9-]*$/.test(name) || !name.includes("-")) {
    return Left(InvalidTagName(name))
  }
  return Right(Object.freeze({ [TAG]: true as const, name }))
}

const RESERVED_CUSTOM_ATTRIBUTE_NAMES = new Set([
  "alt",
  "attributes",
  "autocomplete",
  "autofocus",
  "buttontype",
  "checked",
  "children",
  "class",
  "classname",
  "cols",
  "colspan",
  "contenteditable",
  "dir",
  "disabled",
  "download",
  "draggable",
  "for",
  "height",
  "hidden",
  "htmlfor",
  "href",
  "id",
  "inputtype",
  "key",
  "lang",
  "loading",
  "max",
  "media",
  "min",
  "mimetype",
  "multiple",
  "name",
  "open",
  "pattern",
  "placeholder",
  "readonly",
  "rel",
  "required",
  "role",
  "rows",
  "rowspan",
  "selected",
  "src",
  "step",
  "style",
  "tabindex",
  "target",
  "title",
  "type",
  "value",
  "width",
])

export function attribute(
  name: string,
  value: string
): Either<HtmlBuildError, Attribute> {
  if (!/^[A-Za-z_:][A-Za-z0-9_.:-]*$/.test(name)) {
    return Left(InvalidAttributeName(name))
  }
  const normalized = name.toLowerCase()
  if (
    (normalized.startsWith("data-") || normalized.startsWith("aria-")) &&
    (name !== normalized || normalized.length <= 5)
  ) {
    return Left(InvalidAttributeName(name))
  }
  if (
    normalized.startsWith("on") ||
    normalized === "data-ssrg" ||
    normalized.startsWith("data-ssrg-") ||
    RESERVED_CUSTOM_ATTRIBUTE_NAMES.has(normalized)
  ) {
    return Left(ReservedAttributeName(name))
  }
  return Right(Object.freeze({ [ATTRIBUTE]: true as const, name, value }))
}

const WEB_URL_BASE = "https://seseragi.invalid/"
const ALLOWED_WEB_URL_PROTOCOLS = new Set([
  "http:",
  "https:",
  "mailto:",
  "tel:",
])

export function parseWebUrl(value: string): Either<HtmlBuildError, WebUrl> {
  if (containsAsciiControl(value)) return Left(UnsafeWebUrlScheme(value))

  let parsed: URL
  try {
    parsed = new URL(value, WEB_URL_BASE)
  } catch {
    return Left(UnsafeWebUrlScheme(value))
  }
  if (
    !ALLOWED_WEB_URL_PROTOCOLS.has(parsed.protocol) ||
    parsed.username !== "" ||
    parsed.password !== ""
  ) {
    return Left(UnsafeWebUrlScheme(value))
  }

  return Right(Object.freeze({ [WEB_URL]: true as const, value }))
}

function containsAsciiControl(value: string): boolean {
  for (let index = 0; index < value.length; index += 1) {
    const code = value.charCodeAt(index)
    if (code <= 0x1f || code === 0x7f) return true
  }
  return false
}

export function text<Action = never>(value: string): Html<Action> {
  return Object.freeze({ [HTML_NODE]: "text", value } as const)
}

export function fragment<Action = never>(children: unknown): Html<Action> {
  return Object.freeze({
    [HTML_NODE]: "fragment",
    children: normalizeChildren<Action>(children),
  } as const)
}

// Seseragi has already checked the action type before lowering. When a
// constructor call has no TypeScript inference site, `never` keeps the pure
// tree safely usable at any checked Html<Action> boundary.
type TagFunction = {
  <Action>(
    props: Readonly<{ onClick: Action }> & Readonly<Record<string, unknown>>
  ): Html<Action>
  <Action>(
    props: Readonly<{ onSubmit: Action }> & Readonly<Record<string, unknown>>
  ): Html<Action>
  <Action>(
    props: Readonly<{ onInput: (event: InputEvent) => Action }> &
      Readonly<Record<string, unknown>>
  ): Html<Action>
  <Action>(
    props: Readonly<{ onChange: (event: ChangeEvent) => Action }> &
      Readonly<Record<string, unknown>>
  ): Html<Action>
  <Action>(
    props: Readonly<{ onFileChange: (event: FileChangeEvent) => Action }> &
      Readonly<Record<string, unknown>>
  ): Html<Action>
  <Action>(
    props: Readonly<{ onFocus: Action }> & Readonly<Record<string, unknown>>
  ): Html<Action>
  <Action>(
    props: Readonly<{ onBlur: Action }> & Readonly<Record<string, unknown>>
  ): Html<Action>
  <Action>(
    props: Readonly<{
      onKeyDown: (event: KeyboardEvent) => EventAction<Action>
    }> &
      Readonly<Record<string, unknown>>
  ): Html<Action>
  <Action>(
    props: Readonly<{
      onKeyUp: (event: KeyboardEvent) => EventAction<Action>
    }> &
      Readonly<Record<string, unknown>>
  ): Html<Action>
  <Action>(
    props: Readonly<{
      onMouseDown: (event: MouseEvent) => EventAction<Action>
    }> &
      Readonly<Record<string, unknown>>
  ): Html<Action>
  <Action>(
    props: Readonly<{ onMouseUp: (event: MouseEvent) => EventAction<Action> }> &
      Readonly<Record<string, unknown>>
  ): Html<Action>
  <Action>(
    props: Readonly<{
      onPointerDown: (event: PointerEvent) => EventAction<Action>
    }> &
      Readonly<Record<string, unknown>>
  ): Html<Action>
  <Action>(
    props: Readonly<{
      onPointerUp: (event: PointerEvent) => EventAction<Action>
    }> &
      Readonly<Record<string, unknown>>
  ): Html<Action>
  <Action>(
    props: Readonly<{
      onDoubleClick: (event: MouseEvent) => EventAction<Action>
    }> &
      Readonly<Record<string, unknown>>
  ): Html<Action>
  <Action>(
    props: Readonly<{
      onContextMenu: (event: MouseEvent) => EventAction<Action>
    }> &
      Readonly<Record<string, unknown>>
  ): Html<Action>
  <Action>(
    props: Readonly<{ onScroll: (event: ScrollEvent) => EventAction<Action> }> &
      Readonly<Record<string, unknown>>
  ): Html<Action>
  <Action = never>(props: unknown): Html<Action>
}

function tag(name: string): TagFunction {
  return <Action>(props: unknown): Html<Action> => element(name, props, false)
}

function voidTag(name: string): TagFunction {
  return <Action>(props: unknown): Html<Action> => element(name, props, true)
}

export const div = tag("div")
export const span = tag("span")
export const p = tag("p")
export const main = tag("main")
export const section = tag("section")
export const h1 = tag("h1")
export const h2 = tag("h2")
export const html = tag("html")
export const head = tag("head")
export const body = tag("body")
export const title = tag("title")
export const meta = voidTag("meta")
export const link = voidTag("link")
export const header = tag("header")
export const footer = tag("footer")
export const nav = tag("nav")
export const article = tag("article")
export const aside = tag("aside")
export const h3 = tag("h3")
export const h4 = tag("h4")
export const h5 = tag("h5")
export const h6 = tag("h6")
export const strong = tag("strong")
export const em = tag("em")
export const small = tag("small")
export const code = tag("code")
export const pre = tag("pre")
export const blockquote = tag("blockquote")
export const ul = tag("ul")
export const ol = tag("ol")
export const li = tag("li")
export const br = voidTag("br")
export const hr = voidTag("hr")
export const a = tag("a")
export const img = voidTag("img")
export const picture = tag("picture")
export const source = voidTag("source")
export const video = tag("video")
export const audio = tag("audio")
export const button = tag("button")
export const form = tag("form")
export const label = tag("label")
export const select = tag("select")
export const option = tag("option")
export const fieldset = tag("fieldset")
export const legend = tag("legend")
export const table = tag("table")
export const thead = tag("thead")
export const tbody = tag("tbody")
export const tfoot = tag("tfoot")
export const tr = tag("tr")
export const th = tag("th")
export const td = tag("td")
export const caption = tag("caption")
export const details = tag("details")
export const summary = tag("summary")
export const dialog = tag("dialog")

export function input<Action = never>(props: unknown): Html<Action> {
  return element("input", props, true)
}

export function textarea<Action = never>(props: unknown): Html<Action> {
  const record = expectProps(props)
  return element("textarea", { ...record, children: record.value ?? "" }, false)
}

export function custom<Action = never>(
  value: Tag,
  props: unknown
): Html<Action> {
  return element(expectTag(value).name, props, false)
}

export function renderToString<Action>(value: Html<Action>): string {
  switch (value[HTML_NODE]) {
    case "text":
      return escapeText(value.value)
    case "fragment":
      return value.children.map(renderToString).join("")
    case "element": {
      const attributes = renderAttributes(value.tag, value.props)
      const opening = `<${value.tag}${attributes}>`
      if (value.voidElement) return opening
      return `${opening}${value.children.map(renderToString).join("")}</${value.tag}>`
    }
  }
}

export function renderDocument<Action>(value: Html<Action>): string {
  return `<!doctype html>${renderToString(value)}`
}

/** Runtime-internal DOM adapter snapshot. SSR output never includes markers. */
export function renderForDom<Action>(
  value: Html<Action>,
  eventIdPrefix = ""
): DomRender<Action> {
  const eventHandlers = new Map<string, DomEventHandler<Action>>()
  return Object.freeze({
    html: renderDomNode(value, eventHandlers, eventIdPrefix),
    eventHandlers,
  })
}

function renderDomNode<Action>(
  value: Html<Action>,
  eventHandlers: Map<string, DomEventHandler<Action>>,
  eventIdPrefix: string
): string {
  switch (value[HTML_NODE]) {
    case "text":
      return escapeText(value.value)
    case "fragment":
      return value.children
        .map((child) => renderDomNode(child, eventHandlers, eventIdPrefix))
        .join("")
    case "element": {
      const markers = registerDomEvents(
        value.props,
        eventHandlers,
        eventIdPrefix
      )
      const attributes = renderAttributes(value.tag, value.props, markers)
      const opening = `<${value.tag}${attributes}>`
      if (value.voidElement) return opening
      return `${opening}${value.children
        .map((child) => renderDomNode(child, eventHandlers, eventIdPrefix))
        .join("")}</${value.tag}>`
    }
  }
}

function registerDomEvents<Action>(
  props: Readonly<Record<string, unknown>>,
  eventHandlers: Map<string, DomEventHandler<Action>>,
  eventIdPrefix: string
): Readonly<Record<string, string>> {
  const markers: Record<string, string> = {}
  const register = (
    kind: DomEventHandler<Action>["kind"],
    handler: DomEventHandler<Action>
  ): void => {
    const id = `${eventIdPrefix}${eventHandlers.size}`
    eventHandlers.set(id, Object.freeze(handler))
    markers[kind] = id
  }
  if (Object.hasOwn(props, "onClick")) {
    register("click", {
      kind: "click",
      message: props.onClick as Action,
      preventDefault: optionalEventControl(
        "preventClickDefault",
        props.preventClickDefault
      ),
      stopPropagation: optionalEventControl(
        "stopClickPropagation",
        props.stopClickPropagation
      ),
    })
  }
  if (Object.hasOwn(props, "onFocus")) {
    register("focus", { kind: "focus", message: props.onFocus as Action })
  }
  if (Object.hasOwn(props, "onBlur")) {
    register("blur", { kind: "blur", message: props.onBlur as Action })
  }
  if (Object.hasOwn(props, "onKeyDown")) {
    register("keydown", {
      kind: "keydown",
      map: expectEventMapper<KeyboardEvent, EventAction<Action>>(
        "onKeyDown",
        props.onKeyDown
      ),
    })
  }
  if (Object.hasOwn(props, "onKeyUp")) {
    register("keyup", {
      kind: "keyup",
      map: expectEventMapper<KeyboardEvent, EventAction<Action>>(
        "onKeyUp",
        props.onKeyUp
      ),
    })
  }
  for (const [prop, kind] of [
    ["onMouseDown", "mousedown"],
    ["onMouseUp", "mouseup"],
    ["onDoubleClick", "dblclick"],
    ["onContextMenu", "contextmenu"],
  ] as const) {
    if (Object.hasOwn(props, prop)) {
      register(kind, {
        kind,
        map: expectEventMapper<MouseEvent, EventAction<Action>>(
          prop,
          props[prop]
        ),
      })
    }
  }
  for (const [prop, kind] of [
    ["onPointerDown", "pointerdown"],
    ["onPointerUp", "pointerup"],
  ] as const) {
    if (Object.hasOwn(props, prop)) {
      register(kind, {
        kind,
        map: expectEventMapper<PointerEvent, EventAction<Action>>(
          prop,
          props[prop]
        ),
      })
    }
  }
  if (Object.hasOwn(props, "onScroll")) {
    register("scroll", {
      kind: "scroll",
      map: expectEventMapper<ScrollEvent, EventAction<Action>>(
        "onScroll",
        props.onScroll
      ),
    })
  }
  if (Object.hasOwn(props, "onInput")) {
    register("input", {
      kind: "input",
      map: expectEventMapper<InputEvent, Action>("onInput", props.onInput),
    })
  }
  if (Object.hasOwn(props, "onChange")) {
    register("change", {
      kind: "change",
      map: expectEventMapper<ChangeEvent, Action>("onChange", props.onChange),
    })
  }
  if (Object.hasOwn(props, "onFileChange")) {
    register("file-change", {
      kind: "file-change",
      map: expectEventMapper<FileChangeEvent, Action>(
        "onFileChange",
        props.onFileChange
      ),
    })
  }
  if (Object.hasOwn(props, "onSubmit")) {
    register("submit", {
      kind: "submit",
      message: props.onSubmit as Action,
    })
  }
  return markers
}

function element<Action>(
  name: string,
  value: unknown,
  voidElement: boolean
): Html<Action> {
  const props = expectProps(value)
  if (voidElement && Object.hasOwn(props, "children")) {
    throw new TypeError(`void HTML element ${name} cannot have children`)
  }
  const children = voidElement
    ? (Object.freeze([]) as ReadonlyArray<Html<Action>>)
    : normalizeChildren<Action>(props.children)
  return Object.freeze({
    [HTML_NODE]: "element",
    tag: name,
    props: Object.freeze({ ...props }),
    children,
    voidElement,
  } as const)
}

function normalizeChildren<Action>(
  value: unknown
): ReadonlyArray<Html<Action>> {
  if (value === undefined) return Object.freeze([])
  if (typeof value === "string") return Object.freeze([text<Action>(value)])
  if (isHtml<Action>(value)) return Object.freeze([value])
  if (Array.isArray(value)) {
    if (!value.every((child) => isHtml<Action>(child))) {
      throw new TypeError("HTML child arrays may contain only Html values")
    }
    return Object.freeze([...value]) as ReadonlyArray<Html<Action>>
  }
  if (isList(value)) {
    const children: Html<Action>[] = []
    let cursor: ListValue = value
    while (cursor.tag === "Cons") {
      if (!isHtml<Action>(cursor.head)) {
        throw new TypeError("HTML child lists may contain only Html values")
      }
      children.push(cursor.head)
      cursor = cursor.tail
    }
    return Object.freeze(children)
  }
  throw new TypeError("unsupported HTML children value")
}

function renderAttributes(
  tagName: string,
  props: Readonly<Record<string, unknown>>,
  eventMarkers: Readonly<Record<string, string>> = {}
): string {
  const attributes: string[] = []
  stringAttribute(attributes, "id", props.id)
  stringAttribute(attributes, "class", props.class)
  stringAttribute(attributes, "title", props.title)
  booleanAttribute(attributes, "hidden", props.hidden)
  styleAttribute(attributes, props.style)
  stringAttribute(attributes, "role", props.role)
  integerAttribute(attributes, "tabindex", props.tabIndex)
  stringAttribute(attributes, "lang", props.lang)
  stringAttribute(attributes, "dir", props.dir)
  enumeratedBooleanAttribute(attributes, "draggable", props.draggable)
  enumeratedBooleanAttribute(
    attributes,
    "contenteditable",
    props.contentEditable
  )
  for (const kind of [
    "click",
    "focus",
    "blur",
    "keydown",
    "keyup",
    "mousedown",
    "mouseup",
    "pointerdown",
    "pointerup",
    "dblclick",
    "contextmenu",
    "scroll",
    "input",
    "change",
    "file-change",
    "submit",
  ] as const) {
    const id = eventMarkers[kind]
    if (id !== undefined) {
      attributes.push(`data-ssrg-event-${kind}="${id}"`)
    }
  }

  if (tagName === "button") {
    booleanAttribute(attributes, "disabled", props.disabled)
    stringAttribute(attributes, "type", props.buttonType ?? "button")
    stringAttribute(attributes, "name", props.name)
    stringAttribute(attributes, "value", props.value)
    booleanAttribute(attributes, "autofocus", props.autoFocus)
  }
  if (tagName === "form") {
    stringAttribute(attributes, "name", props.name)
    stringAttribute(attributes, "autocomplete", props.autoComplete)
  }
  if (tagName === "input") {
    stringAttribute(attributes, "value", props.value)
    booleanAttribute(attributes, "checked", props.checked)
    stringAttribute(attributes, "name", props.name)
    booleanAttribute(attributes, "disabled", props.disabled)
    booleanAttribute(attributes, "required", props.required)
    booleanAttribute(attributes, "readonly", props.readOnly)
    booleanAttribute(attributes, "multiple", props.multiple)
    stringAttribute(attributes, "placeholder", props.placeholder)
    stringAttribute(attributes, "autocomplete", props.autoComplete)
    booleanAttribute(attributes, "autofocus", props.autoFocus)
    stringAttribute(attributes, "min", props.min)
    stringAttribute(attributes, "max", props.max)
    stringAttribute(attributes, "step", props.step)
    stringAttribute(attributes, "pattern", props.pattern)
    stringAttribute(attributes, "type", props.inputType ?? "text")
  }
  if (tagName === "textarea") {
    stringAttribute(attributes, "name", props.name)
    booleanAttribute(attributes, "disabled", props.disabled)
    booleanAttribute(attributes, "required", props.required)
    booleanAttribute(attributes, "readonly", props.readOnly)
    stringAttribute(attributes, "placeholder", props.placeholder)
    stringAttribute(attributes, "autocomplete", props.autoComplete)
    booleanAttribute(attributes, "autofocus", props.autoFocus)
    integerAttribute(attributes, "rows", props.rows)
    integerAttribute(attributes, "cols", props.cols)
  }
  if (tagName === "label") {
    stringAttribute(attributes, "for", props.htmlFor)
  }
  if (tagName === "a") {
    webUrlAttribute(attributes, "href", props.href)
    stringAttribute(attributes, "target", props.target)
    stringAttribute(attributes, "rel", props.rel)
    booleanAttribute(attributes, "download", props.download)
  }
  if (tagName === "img") {
    webUrlAttribute(attributes, "src", props.src)
    stringAttribute(attributes, "alt", props.alt)
    integerAttribute(attributes, "width", props.width)
    integerAttribute(attributes, "height", props.height)
    stringAttribute(attributes, "loading", props.loading)
  }
  if (tagName === "source") {
    webUrlAttribute(attributes, "src", props.src)
    stringAttribute(attributes, "media", props.media)
    stringAttribute(attributes, "type", props.mimeType)
  }
  if (tagName === "video" || tagName === "audio") {
    webUrlAttribute(attributes, "src", props.src)
  }
  if (tagName === "video") {
    integerAttribute(attributes, "width", props.width)
    integerAttribute(attributes, "height", props.height)
  }
  if (tagName === "link") {
    stringAttribute(attributes, "rel", props.rel)
    webUrlAttribute(attributes, "href", props.href)
  }
  if (tagName === "select") {
    stringAttribute(attributes, "name", props.name)
    stringAttribute(attributes, "value", props.value)
    booleanAttribute(attributes, "disabled", props.disabled)
    booleanAttribute(attributes, "required", props.required)
    booleanAttribute(attributes, "multiple", props.multiple)
    booleanAttribute(attributes, "autofocus", props.autoFocus)
  }
  if (tagName === "option") {
    stringAttribute(attributes, "value", props.value)
    booleanAttribute(attributes, "selected", props.selected)
    booleanAttribute(attributes, "disabled", props.disabled)
  }
  if (tagName === "th" || tagName === "td") {
    integerAttribute(attributes, "colspan", props.colSpan)
    integerAttribute(attributes, "rowspan", props.rowSpan)
  }
  if (tagName === "details" || tagName === "dialog") {
    booleanAttribute(attributes, "open", props.open)
  }
  customAttributes(attributes, props.attributes)
  return attributes.length === 0 ? "" : ` ${attributes.join(" ")}`
}

export function messageFromDomEvent<Action>(
  handler: DomEventHandler<Action>,
  target: unknown,
  event: unknown = target
): Action {
  const resolution = resolveDomEvent(handler, target, event)
  if (resolution.kind === "ignore") {
    throw new TypeError("ignored DOM events do not contain an Action")
  }
  return resolution.action
}

export function resolveDomEvent<Action>(
  handler: DomEventHandler<Action>,
  target: unknown,
  event: unknown = target
): DomEventResolution<Action> {
  switch (handler.kind) {
    case "click":
      return dispatchResolution(
        handler.message,
        handler.preventDefault,
        handler.stopPropagation
      )
    case "focus":
    case "blur":
      return dispatchResolution(handler.message)
    case "submit":
      return dispatchResolution(handler.message, true)
    case "keydown":
    case "keyup":
      return resolveEventAction(handler.map(keyboardEventSnapshot(event)))
    case "mousedown":
    case "mouseup":
    case "dblclick":
    case "contextmenu":
      return resolveEventAction(handler.map(mouseEventSnapshot(event)))
    case "pointerdown":
    case "pointerup":
      return resolveEventAction(handler.map(pointerEventSnapshot(event)))
    case "scroll":
      return resolveEventAction(handler.map(scrollEventSnapshot(target)))
    case "input":
      return dispatchResolution(
        handler.map(
          Object.freeze({ value: eventTargetString("value", target) })
        )
      )
    case "change":
      return dispatchResolution(
        handler.map(
          Object.freeze({
            value: eventTargetString("value", target),
            checked: eventTargetBoolean("checked", target),
          })
        )
      )
    case "file-change":
      return dispatchResolution(
        handler.map(Object.freeze({ files: eventTargetFiles(target) }))
      )
  }
}

function eventTargetFiles(target: unknown): ReadonlyArray<File> {
  if (typeof target !== "object" || target === null || !("files" in target)) {
    return Object.freeze([])
  }
  const files = (target as { readonly files?: FileList | null }).files
  if (files === undefined || files === null) return Object.freeze([])
  return Object.freeze(Array.from(files, wrapFile))
}

function dispatchResolution<Action>(
  action: Action,
  preventDefault = false,
  stopPropagation = false
): DomEventResolution<Action> {
  return Object.freeze({
    kind: "dispatch",
    action,
    preventDefault,
    stopPropagation,
  })
}

function resolveEventAction<Action>(
  value: EventAction<Action>
): DomEventResolution<Action> {
  if (typeof value !== "object" || value === null) {
    throw new TypeError("DOM event mappers must return html.EventAction")
  }
  switch (value.tag) {
    case "IgnoreEvent":
      return Object.freeze({
        kind: "ignore",
        preventDefault: false,
        stopPropagation: false,
      })
    case "Dispatch":
      return dispatchResolution(value.value)
    case "DispatchPreventDefault":
      return dispatchResolution(value.value, true)
    case "DispatchStopPropagation":
      return dispatchResolution(value.value, false, true)
    case "DispatchPreventDefaultAndStop":
      return dispatchResolution(value.value, true, true)
    default:
      throw new TypeError("DOM event mapper returned an unknown EventAction")
  }
}

function mouseEventSnapshot(event: unknown): MouseEvent {
  return Object.freeze({
    button: eventTargetInt("button", event),
    clientX: eventTargetNumber("clientX", event),
    clientY: eventTargetNumber("clientY", event),
    ...modifierSnapshot(event),
  })
}

function pointerEventSnapshot(event: unknown): PointerEvent {
  return Object.freeze({
    pointerId: eventTargetInt("pointerId", event),
    pointerType: eventTargetString("pointerType", event),
    isPrimary: eventTargetBoolean("isPrimary", event),
    button: eventTargetInt("button", event),
    clientX: eventTargetNumber("clientX", event),
    clientY: eventTargetNumber("clientY", event),
    pressure: eventTargetNumber("pressure", event),
    ...modifierSnapshot(event),
  })
}

function scrollEventSnapshot(target: unknown): ScrollEvent {
  return Object.freeze({
    scrollLeft: eventTargetNumber("scrollLeft", target),
    scrollTop: eventTargetNumber("scrollTop", target),
  })
}

function modifierSnapshot(event: unknown) {
  return {
    altKey: eventTargetBoolean("altKey", event),
    controlKey: eventTargetBoolean("ctrlKey", event),
    metaKey: eventTargetBoolean("metaKey", event),
    shiftKey: eventTargetBoolean("shiftKey", event),
  } as const
}

function keyboardEventSnapshot(event: unknown): KeyboardEvent {
  return Object.freeze({
    key: eventTargetString("key", event),
    code: eventTargetString("code", event),
    repeat: eventTargetBoolean("repeat", event),
    altKey: eventTargetBoolean("altKey", event),
    controlKey: eventTargetBoolean("ctrlKey", event),
    metaKey: eventTargetBoolean("metaKey", event),
    shiftKey: eventTargetBoolean("shiftKey", event),
  })
}

export function domEventPreventsDefault(
  handler: DomEventHandler<unknown>
): boolean {
  return (
    handler.kind === "submit" ||
    (handler.kind === "click" && handler.preventDefault)
  )
}

export function domEventStopsPropagation(
  handler: DomEventHandler<unknown>
): boolean {
  return handler.kind === "click" && handler.stopPropagation
}

function styleAttribute(output: string[], value: unknown): void {
  if (value === undefined) return
  if (!isStyle(value)) {
    throw new TypeError("HTML style must be created with html.style")
  }
  output.push(`style="${escapeAttribute(value.cssText)}"`)
}

function stringAttribute(output: string[], name: string, value: unknown): void {
  if (value === undefined) return
  if (typeof value !== "string") {
    throw new TypeError(`HTML attribute ${name} must be a string`)
  }
  output.push(`${name}="${escapeAttribute(value)}"`)
}

function webUrlAttribute(output: string[], name: string, value: unknown): void {
  if (value === undefined) return
  if (!isWebUrl(value)) {
    throw new TypeError(
      `HTML URL attribute ${name} must be created with html.parseWebUrl`
    )
  }
  output.push(`${name}="${escapeAttribute(value.value)}"`)
}

function booleanAttribute(
  output: string[],
  name: string,
  value: unknown
): void {
  if (value === true) output.push(name)
}

function enumeratedBooleanAttribute(
  output: string[],
  name: string,
  value: unknown
): void {
  if (value === undefined) return
  if (typeof value !== "boolean") {
    throw new TypeError(`HTML attribute ${name} must be a boolean`)
  }
  output.push(`${name}="${String(value)}"`)
}

function integerAttribute(
  output: string[],
  name: string,
  value: unknown
): void {
  if (value === undefined) return
  if (typeof value !== "number" || !Number.isSafeInteger(value)) {
    throw new TypeError(`HTML attribute ${name} must be an Int`)
  }
  output.push(`${name}="${value}"`)
}

function customAttributes(output: string[], value: unknown): void {
  if (value === undefined) return
  if (!Array.isArray(value)) {
    throw new TypeError("HTML custom attributes must be an Array")
  }
  const names = new Set<string>()
  for (const item of value) {
    if (!isAttribute(item)) {
      throw new TypeError(
        "HTML custom attributes must be created with html.attribute"
      )
    }
    const normalized = item.name.toLowerCase()
    if (names.has(normalized)) {
      throw new TypeError(`duplicate HTML custom attribute ${item.name}`)
    }
    names.add(normalized)
    output.push(`${item.name}="${escapeAttribute(item.value)}"`)
  }
}

function escapeText(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
}

function escapeAttribute(value: string): string {
  return escapeText(value).replaceAll('"', "&quot;").replaceAll("'", "&#39;")
}

function expectEventMapper<Event, Action>(
  name: string,
  value: unknown
): (event: Event) => Action {
  if (typeof value !== "function") {
    throw new TypeError(`HTML event ${name} must be a function`)
  }
  return value as (event: Event) => Action
}

function optionalEventControl(name: string, value: unknown): boolean {
  if (value === undefined) return false
  if (typeof value !== "boolean") {
    throw new TypeError(`HTML event control ${name} must be a boolean`)
  }
  return value
}

function eventTargetNumber(name: string, target: unknown): number {
  const value = eventTargetProperty(name, target)
  if (typeof value !== "number" || !Number.isFinite(value)) {
    throw new TypeError(`DOM event source ${name} must be a finite number`)
  }
  return value
}

function eventTargetInt(name: string, target: unknown): number {
  const value = eventTargetNumber(name, target)
  if (!Number.isSafeInteger(value)) {
    throw new TypeError(`DOM event source ${name} must be a safe integer`)
  }
  return value === 0 ? 0 : value
}

function eventTargetString(name: string, target: unknown): string {
  const value = eventTargetProperty(name, target)
  if (typeof value !== "string") {
    throw new TypeError(`DOM event target ${name} must be a string`)
  }
  return value
}

function eventTargetBoolean(name: string, target: unknown): boolean {
  const value = eventTargetProperty(name, target)
  if (typeof value !== "boolean") {
    throw new TypeError(`DOM event target ${name} must be a boolean`)
  }
  return value
}

function eventTargetProperty(name: string, target: unknown): unknown {
  if ((typeof target !== "object" && typeof target !== "function") || !target) {
    throw new TypeError("DOM event source must expose snapshot state")
  }
  return Reflect.get(target, name)
}

function expectProps(value: unknown): Readonly<Record<string, unknown>> {
  return expectRecord(value, "HTML tag props")
}

function expectRecord(
  value: unknown,
  label: string
): Readonly<Record<string, unknown>> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new TypeError(`${label} must be a record`)
  }
  return value as Readonly<Record<string, unknown>>
}

function expectStyleValue(name: string, value: unknown): string {
  if (typeof value !== "string") {
    throw new TypeError(`HTML style ${name} must be a string`)
  }
  return value
}

function camelToKebab(value: string): string {
  return value.replaceAll(
    /[A-Z]/g,
    (character) => `-${character.toLowerCase()}`
  )
}

function isStyle(value: unknown): value is Style {
  return typeof value === "object" && value !== null && STYLE in value
}

function expectTag(value: unknown): Tag {
  if (typeof value !== "object" || value === null || !(TAG in value)) {
    throw new TypeError("custom HTML tags must be created with html.customTag")
  }
  return value as Tag
}

function isAttribute(value: unknown): value is Attribute {
  return typeof value === "object" && value !== null && ATTRIBUTE in value
}

function isWebUrl(value: unknown): value is WebUrl {
  return typeof value === "object" && value !== null && WEB_URL in value
}

function isHtml<Action>(value: unknown): value is Html<Action> {
  return typeof value === "object" && value !== null && HTML_NODE in value
}

type ListValue =
  | Readonly<{ tag: "Empty" }>
  | Readonly<{ tag: "Cons"; head: unknown; tail: ListValue }>

function isList(value: unknown): value is ListValue {
  return (
    typeof value === "object" &&
    value !== null &&
    (Reflect.get(value, "tag") === "Empty" ||
      Reflect.get(value, "tag") === "Cons")
  )
}
