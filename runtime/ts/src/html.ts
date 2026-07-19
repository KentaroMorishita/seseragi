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

export type DomRender<Message> = Readonly<{
  readonly html: string
  readonly clickMessages: ReadonlyMap<string, Message>
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

export function input<Message = never>(props: unknown): Html<Message> {
  return element("input", props, true)
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
  const clickMessages = new Map<string, Message>()
  return Object.freeze({
    html: renderDomNode(value, clickMessages),
    clickMessages,
  })
}

function renderDomNode<Message>(
  value: Html<Message>,
  clickMessages: Map<string, Message>
): string {
  switch (value[HTML_NODE]) {
    case "text":
      return escapeText(value.value)
    case "fragment":
      return value.children
        .map((child) => renderDomNode(child, clickMessages))
        .join("")
    case "element": {
      let clickId: string | undefined
      if (Object.hasOwn(value.props, "onClick")) {
        clickId = String(clickMessages.size)
        clickMessages.set(clickId, value.props.onClick as Message)
      }
      const attributes = renderAttributes(value.tag, value.props, clickId)
      const opening = `<${value.tag}${attributes}>`
      if (value.voidElement) return opening
      return `${opening}${value.children
        .map((child) => renderDomNode(child, clickMessages))
        .join("")}</${value.tag}>`
    }
  }
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
  clickId?: string
): string {
  const attributes: string[] = []
  stringAttribute(attributes, "id", props.id)
  stringAttribute(attributes, "class", props.className)
  stringAttribute(attributes, "title", props.title)
  booleanAttribute(attributes, "hidden", props.hidden)
  styleAttribute(attributes, props.style)
  if (clickId !== undefined) {
    attributes.push(`data-ssrg-click="${clickId}"`)
  }

  if (tagName === "button") {
    booleanAttribute(attributes, "disabled", props.disabled)
    stringAttribute(attributes, "type", props.buttonType ?? "button")
  }
  if (tagName === "input") {
    stringAttribute(attributes, "value", props.value)
    booleanAttribute(attributes, "checked", props.checked)
    booleanAttribute(attributes, "disabled", props.disabled)
    stringAttribute(attributes, "placeholder", props.placeholder)
    stringAttribute(attributes, "type", props.inputType ?? "text")
  }
  return attributes.length === 0 ? "" : ` ${attributes.join(" ")}`
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
