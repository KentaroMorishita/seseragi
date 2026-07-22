const HTML_NODE = Symbol("seseragi.html")
const STYLE = Symbol("seseragi.style")

type PhantomMessage<Message> = {
  readonly __message?: Message
}

type TextNode<Message> = PhantomMessage<Message> &
  Readonly<{
    readonly [HTML_NODE]: "text"
    readonly value: string
  }>

type FragmentNode<Message> = PhantomMessage<Message> &
  Readonly<{
    readonly [HTML_NODE]: "fragment"
    readonly children: ReadonlyArray<Html<Message>>
  }>

type ElementNode<Message> = PhantomMessage<Message> &
  Readonly<{
    readonly [HTML_NODE]: "element"
    readonly tag: string
    readonly props: Readonly<Record<string, unknown>>
    readonly children: ReadonlyArray<Html<Message>>
    readonly voidElement: boolean
  }>

/** Immutable pure HTML tree. The message parameter is a compile-time phantom. */
export type Html<Message> =
  | TextNode<Message>
  | FragmentNode<Message>
  | ElementNode<Message>

/** Immutable text-input snapshot. It never exposes the host DOM event. */
export type InputEvent = Readonly<{
  readonly value: string
}>

/** Immutable change snapshot shared by text and checked controls. */
export type ChangeEvent = Readonly<{
  readonly value: string
  readonly checked: boolean
}>

export type DomEventHandler<Message> =
  | Readonly<{ readonly kind: "click"; readonly message: Message }>
  | Readonly<{
      readonly kind: "input"
      readonly map: (event: InputEvent) => Message
    }>
  | Readonly<{
      readonly kind: "change"
      readonly map: (event: ChangeEvent) => Message
    }>
  | Readonly<{ readonly kind: "submit"; readonly message: Message }>

export type DomRender<Message> = Readonly<{
  readonly html: string
  readonly eventHandlers: ReadonlyMap<string, DomEventHandler<Message>>
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

export function text<Message = never>(value: string): Html<Message> {
  return Object.freeze({ [HTML_NODE]: "text", value } as const)
}

export function fragment<Message = never>(children: unknown): Html<Message> {
  return Object.freeze({
    [HTML_NODE]: "fragment",
    children: normalizeChildren<Message>(children),
  } as const)
}

// Seseragi has already checked the message type before lowering. When a
// constructor call has no TypeScript inference site, `never` keeps the pure
// tree safely usable at any checked Html<Message> boundary.
type TagFunction = {
  <Message>(
    props: Readonly<{ onClick: Message }> & Readonly<Record<string, unknown>>
  ): Html<Message>
  <Message>(
    props: Readonly<{ onSubmit: Message }> & Readonly<Record<string, unknown>>
  ): Html<Message>
  <Message>(
    props: Readonly<{ onInput: (event: InputEvent) => Message }> &
      Readonly<Record<string, unknown>>
  ): Html<Message>
  <Message>(
    props: Readonly<{ onChange: (event: ChangeEvent) => Message }> &
      Readonly<Record<string, unknown>>
  ): Html<Message>
  <Message = never>(props: unknown): Html<Message>
}

function tag(name: string): TagFunction {
  return <Message>(props: unknown): Html<Message> => element(name, props, false)
}

export const div = tag("div")
export const span = tag("span")
export const p = tag("p")
export const main = tag("main")
export const section = tag("section")
export const h1 = tag("h1")
export const h2 = tag("h2")
export const button = tag("button")
export const form = tag("form")
export const label = tag("label")

export function input<Message = never>(props: unknown): Html<Message> {
  return element("input", props, true)
}

export function textarea<Message = never>(props: unknown): Html<Message> {
  const record = expectProps(props)
  return element("textarea", { ...record, children: record.value ?? "" }, false)
}

export function renderToString<Message>(value: Html<Message>): string {
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

export function renderDocument<Message>(value: Html<Message>): string {
  return `<!doctype html>${renderToString(value)}`
}

/** Runtime-internal DOM adapter snapshot. SSR output never includes markers. */
export function renderForDom<Message>(
  value: Html<Message>
): DomRender<Message> {
  const eventHandlers = new Map<string, DomEventHandler<Message>>()
  return Object.freeze({
    html: renderDomNode(value, eventHandlers),
    eventHandlers,
  })
}

function renderDomNode<Message>(
  value: Html<Message>,
  eventHandlers: Map<string, DomEventHandler<Message>>
): string {
  switch (value[HTML_NODE]) {
    case "text":
      return escapeText(value.value)
    case "fragment":
      return value.children
        .map((child) => renderDomNode(child, eventHandlers))
        .join("")
    case "element": {
      const markers = registerDomEvents(value.props, eventHandlers)
      const attributes = renderAttributes(value.tag, value.props, markers)
      const opening = `<${value.tag}${attributes}>`
      if (value.voidElement) return opening
      return `${opening}${value.children
        .map((child) => renderDomNode(child, eventHandlers))
        .join("")}</${value.tag}>`
    }
  }
}

function registerDomEvents<Message>(
  props: Readonly<Record<string, unknown>>,
  eventHandlers: Map<string, DomEventHandler<Message>>
): Readonly<Record<string, string>> {
  const markers: Record<string, string> = {}
  const register = (
    kind: DomEventHandler<Message>["kind"],
    handler: DomEventHandler<Message>
  ): void => {
    const id = String(eventHandlers.size)
    eventHandlers.set(id, Object.freeze(handler))
    markers[kind] = id
  }
  if (Object.hasOwn(props, "onClick")) {
    register("click", { kind: "click", message: props.onClick as Message })
  }
  if (Object.hasOwn(props, "onInput")) {
    register("input", {
      kind: "input",
      map: expectEventMapper<InputEvent, Message>("onInput", props.onInput),
    })
  }
  if (Object.hasOwn(props, "onChange")) {
    register("change", {
      kind: "change",
      map: expectEventMapper<ChangeEvent, Message>("onChange", props.onChange),
    })
  }
  if (Object.hasOwn(props, "onSubmit")) {
    register("submit", {
      kind: "submit",
      message: props.onSubmit as Message,
    })
  }
  return markers
}

function element<Message>(
  name: string,
  value: unknown,
  voidElement: boolean
): Html<Message> {
  const props = expectProps(value)
  const children = voidElement
    ? (Object.freeze([]) as ReadonlyArray<Html<Message>>)
    : normalizeChildren<Message>(props.children)
  return Object.freeze({
    [HTML_NODE]: "element",
    tag: name,
    props: Object.freeze({ ...props }),
    children,
    voidElement,
  } as const)
}

function normalizeChildren<Message>(
  value: unknown
): ReadonlyArray<Html<Message>> {
  if (value === undefined) return Object.freeze([])
  if (typeof value === "string") return Object.freeze([text<Message>(value)])
  if (isHtml<Message>(value)) return Object.freeze([value])
  if (Array.isArray(value)) {
    if (!value.every((child) => isHtml<Message>(child))) {
      throw new TypeError("HTML child arrays may contain only Html values")
    }
    return Object.freeze([...value]) as ReadonlyArray<Html<Message>>
  }
  if (isList(value)) {
    const children: Html<Message>[] = []
    let cursor: ListValue = value
    while (cursor.tag === "Cons") {
      if (!isHtml<Message>(cursor.head)) {
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
  stringAttribute(attributes, "class", props.className)
  stringAttribute(attributes, "title", props.title)
  booleanAttribute(attributes, "hidden", props.hidden)
  styleAttribute(attributes, props.style)
  for (const kind of ["click", "input", "change", "submit"] as const) {
    const id = eventMarkers[kind]
    if (id !== undefined) {
      attributes.push(`data-ssrg-event-${kind}="${id}"`)
    }
  }

  if (tagName === "button") {
    booleanAttribute(attributes, "disabled", props.disabled)
    stringAttribute(attributes, "type", props.buttonType ?? "button")
  }
  if (tagName === "input") {
    stringAttribute(attributes, "value", props.value)
    booleanAttribute(attributes, "checked", props.checked)
    stringAttribute(attributes, "name", props.name)
    booleanAttribute(attributes, "disabled", props.disabled)
    booleanAttribute(attributes, "required", props.required)
    stringAttribute(attributes, "placeholder", props.placeholder)
    stringAttribute(attributes, "type", props.inputType ?? "text")
  }
  if (tagName === "textarea") {
    stringAttribute(attributes, "name", props.name)
    booleanAttribute(attributes, "disabled", props.disabled)
    booleanAttribute(attributes, "required", props.required)
    stringAttribute(attributes, "placeholder", props.placeholder)
  }
  if (tagName === "label") {
    stringAttribute(attributes, "for", props.htmlFor)
  }
  return attributes.length === 0 ? "" : ` ${attributes.join(" ")}`
}

export function messageFromDomEvent<Message>(
  handler: DomEventHandler<Message>,
  target: unknown
): Message {
  switch (handler.kind) {
    case "click":
    case "submit":
      return handler.message
    case "input":
      return handler.map(
        Object.freeze({ value: eventTargetString("value", target) })
      )
    case "change":
      return handler.map(
        Object.freeze({
          value: eventTargetString("value", target),
          checked: eventTargetBoolean("checked", target),
        })
      )
  }
}

export function domEventPreventsDefault(
  handler: DomEventHandler<unknown>
): boolean {
  return handler.kind === "submit"
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

function booleanAttribute(
  output: string[],
  name: string,
  value: unknown
): void {
  if (value === true) output.push(name)
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

function expectEventMapper<Event, Message>(
  name: string,
  value: unknown
): (event: Event) => Message {
  if (typeof value !== "function") {
    throw new TypeError(`HTML event ${name} must be a function`)
  }
  return value as (event: Event) => Message
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
    throw new TypeError("DOM event target must expose form control state")
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

function isHtml<Message>(value: unknown): value is Html<Message> {
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
