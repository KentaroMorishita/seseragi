const HTML_NODE = Symbol("seseragi.html")
const STYLE = Symbol("seseragi.style")

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

export type DomEventHandler<Action> =
  | Readonly<{ readonly kind: "click"; readonly message: Action }>
  | Readonly<{
      readonly kind: "input"
      readonly map: (event: InputEvent) => Action
    }>
  | Readonly<{
      readonly kind: "change"
      readonly map: (event: ChangeEvent) => Action
    }>
  | Readonly<{ readonly kind: "submit"; readonly message: Action }>

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
  <Action = never>(props: unknown): Html<Action>
}

function tag(name: string): TagFunction {
  return <Action>(props: unknown): Html<Action> => element(name, props, false)
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

export function input<Action = never>(props: unknown): Html<Action> {
  return element("input", props, true)
}

export function textarea<Action = never>(props: unknown): Html<Action> {
  const record = expectProps(props)
  return element("textarea", { ...record, children: record.value ?? "" }, false)
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
export function renderForDom<Action>(value: Html<Action>): DomRender<Action> {
  const eventHandlers = new Map<string, DomEventHandler<Action>>()
  return Object.freeze({
    html: renderDomNode(value, eventHandlers),
    eventHandlers,
  })
}

function renderDomNode<Action>(
  value: Html<Action>,
  eventHandlers: Map<string, DomEventHandler<Action>>
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

function registerDomEvents<Action>(
  props: Readonly<Record<string, unknown>>,
  eventHandlers: Map<string, DomEventHandler<Action>>
): Readonly<Record<string, string>> {
  const markers: Record<string, string> = {}
  const register = (
    kind: DomEventHandler<Action>["kind"],
    handler: DomEventHandler<Action>
  ): void => {
    const id = String(eventHandlers.size)
    eventHandlers.set(id, Object.freeze(handler))
    markers[kind] = id
  }
  if (Object.hasOwn(props, "onClick")) {
    register("click", { kind: "click", message: props.onClick as Action })
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

export function messageFromDomEvent<Action>(
  handler: DomEventHandler<Action>,
  target: unknown
): Action {
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

function expectEventMapper<Event, Action>(
  name: string,
  value: unknown
): (event: Event) => Action {
  if (typeof value !== "function") {
    throw new TypeError(`HTML event ${name} must be a function`)
  }
  return value as (event: Event) => Action
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
