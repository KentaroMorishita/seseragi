const HTML_NODE = Symbol("seseragi.html")

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

export function text<Message>(value: string): Html<Message> {
  return Object.freeze({ [HTML_NODE]: "text", value } as const)
}

export function fragment<Message>(children: unknown): Html<Message> {
  return Object.freeze({
    [HTML_NODE]: "fragment",
    children: normalizeChildren<Message>(children),
  } as const)
}

type TagFunction = <Message>(props: unknown) => Html<Message>

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

export function input<Message>(props: unknown): Html<Message> {
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
  props: Readonly<Record<string, unknown>>
): string {
  const attributes: string[] = []
  stringAttribute(attributes, "id", props.id)
  stringAttribute(attributes, "class", props.className)
  stringAttribute(attributes, "title", props.title)
  booleanAttribute(attributes, "hidden", props.hidden)

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
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new TypeError("HTML tag props must be a record")
  }
  return value as Readonly<Record<string, unknown>>
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
