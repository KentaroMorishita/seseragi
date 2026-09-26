import assert from "node:assert/strict"
import type {
  DocInline,
  PageMetadata,
  ParsedDocBlock,
  ParsedPage,
} from "./model"

const directiveKinds = new Set<ParsedDocBlock["kind"]>([
  "example",
  "from-typescript",
  "design-rationale",
  "common-mistake",
  "warning",
  "availability",
  "api-reference",
])

const emptyBlock = (
  values: Pick<ParsedDocBlock, "kind"> & Partial<ParsedDocBlock>
): ParsedDocBlock => ({
  inlines: [],
  level: 0,
  ordered: false,
  items: [],
  title: "",
  text: "",
  source: "",
  language: "",
  reference: "",
  ...values,
})

function parseInline(source: string): DocInline[] {
  const result: DocInline[] = []
  const pattern =
    /(`[^`\n]+`|\*\*[^*\n]+\*\*|\*[^*\n]+\*|\[[^\]\n]+\]\([^\s)]+\))/gu
  let index = 0
  for (const match of source.matchAll(pattern)) {
    if (match.index > index)
      result.push({
        kind: "text",
        text: source.slice(index, match.index),
        href: "",
      })
    const token = match[0]
    if (token.startsWith("`"))
      result.push({ kind: "code", text: token.slice(1, -1), href: "" })
    else if (token.startsWith("**"))
      result.push({ kind: "strong", text: token.slice(2, -2), href: "" })
    else if (token.startsWith("*"))
      result.push({ kind: "emphasis", text: token.slice(1, -1), href: "" })
    else {
      const link = token.match(/^\[([^\]]+)\]\(([^)]+)\)$/u)
      assert.ok(link, "Invalid Markdown link")
      result.push({ kind: "link", text: link[1], href: link[2] })
    }
    index = match.index + token.length
  }
  if (index < source.length)
    result.push({ kind: "text", text: source.slice(index), href: "" })
  assert.ok(
    !result.some(({ kind, text }) => kind === "text" && text.includes("](")),
    "Malformed Markdown link"
  )
  return result
}

function startsBlock(line: string): boolean {
  return (
    /^#{1,3}\s+/u.test(line) ||
    /^(```|~~~)/u.test(line) ||
    /^:::/u.test(line) ||
    /^(- |\d+\. )/u.test(line)
  )
}

function parseDirective(
  lines: string[],
  start: number,
  path: string
): { block: ParsedDocBlock; next: number } {
  const opening = lines[start].match(/^:::([a-z-]+)(?:\s+(\{.*\}))?$/u)
  assert.ok(opening, `${path}:${start + 1}: invalid directive opening`)
  const kind = opening[1] as ParsedDocBlock["kind"]
  assert.ok(
    directiveKinds.has(kind),
    `${path}:${start + 1}: unknown directive ${kind}`
  )
  let attributes: Record<string, unknown> = {}
  if (opening[2]) {
    const decoded: unknown = JSON.parse(opening[2])
    assert.ok(
      decoded && typeof decoded === "object" && !Array.isArray(decoded),
      `${path}:${start + 1}: directive attributes must be an object`
    )
    attributes = decoded as Record<string, unknown>
  }
  const body: string[] = []
  let index = start + 1
  while (index < lines.length && lines[index] !== ":::") {
    assert.ok(
      !lines[index].startsWith(":::"),
      `${path}:${index + 1}: nested directives are not supported`
    )
    body.push(lines[index])
    index += 1
  }
  assert.ok(index < lines.length, `${path}:${start + 1}: unclosed directive`)

  const allowed =
    kind === "example"
      ? new Set(["title", "source"])
      : kind === "api-reference"
        ? new Set(["title", "identity"])
        : new Set(["title"])
  for (const key of Object.keys(attributes))
    assert.ok(
      allowed.has(key),
      `${path}:${start + 1}: unknown ${kind} attribute ${key}`
    )
  const titleValue = attributes.title ?? ""
  assert.equal(
    typeof titleValue,
    "string",
    `${path}:${start + 1}: title must be text`
  )
  const title = titleValue as string
  const text = body.join("\n").trim()

  if (kind === "example") {
    assert.equal(text, "", `${path}:${start + 1}: example body must be empty`)
    const source = attributes.source
    assert.equal(
      typeof source,
      "string",
      `${path}:${start + 1}: example source is required`
    )
    return {
      block: emptyBlock({ kind, title, source: source as string }),
      next: index + 1,
    }
  }
  if (kind === "api-reference") {
    assert.equal(
      text,
      "",
      `${path}:${start + 1}: api-reference body must be empty`
    )
    const identity = attributes.identity
    assert.equal(
      typeof identity,
      "string",
      `${path}:${start + 1}: api-reference identity is required`
    )
    return {
      block: emptyBlock({
        kind,
        title,
        reference: identity as string,
      }),
      next: index + 1,
    }
  }
  assert.ok(text.length > 0, `${path}:${start + 1}: ${kind} body is required`)
  return {
    block: emptyBlock({ kind, title, inlines: parseInline(text) }),
    next: index + 1,
  }
}

export function parseAuthoringFile(source: string, path: string): ParsedPage {
  const normalized = source.replace(/\r\n?/gu, "\n")
  assert.ok(
    normalized.startsWith("---json\n"),
    `${path}: missing ---json front matter`
  )
  const boundary = normalized.indexOf("\n---\n", 8)
  assert.ok(boundary !== -1, `${path}: unclosed front matter`)
  const metadata = JSON.parse(normalized.slice(8, boundary)) as PageMetadata
  const lines = normalized
    .slice(boundary + 5)
    .trim()
    .split("\n")
  const titleLine = lines.shift() ?? ""
  const title = titleLine.match(/^#\s+(.+)$/u)
  assert.ok(title, `${path}: body must start with one H1`)
  const blocks: ParsedDocBlock[] = []
  let index = 0

  while (index < lines.length) {
    const line = lines[index]
    if (line.trim() === "") {
      index += 1
      continue
    }
    assert.ok(
      !/^#\s+/u.test(line),
      `${path}:${index + 2}: only one H1 is allowed`
    )
    assert.ok(
      !/^\s*</u.test(line),
      `${path}:${index + 2}: raw HTML is not allowed`
    )

    const heading = line.match(/^(#{2,3})\s+(.+)$/u)
    if (heading) {
      blocks.push(
        emptyBlock({
          kind: "heading",
          level: heading[1].length,
          inlines: parseInline(heading[2]),
        })
      )
      index += 1
      continue
    }

    const fence = line.match(/^(```|~~~)([a-z0-9-]*)$/u)
    if (fence) {
      const content: string[] = []
      index += 1
      while (index < lines.length && lines[index] !== fence[1]) {
        content.push(lines[index])
        index += 1
      }
      assert.ok(index < lines.length, `${path}: unclosed code fence`)
      assert.notEqual(
        fence[2],
        "seseragi",
        `${path}: Seseragi code must use a canonical example directive`
      )
      blocks.push(
        emptyBlock({
          kind: "code-block",
          text: content.join("\n"),
          language: fence[2],
        })
      )
      index += 1
      continue
    }

    if (line.startsWith(":::")) {
      const directive = parseDirective(lines, index, path)
      blocks.push(directive.block)
      index = directive.next
      continue
    }

    const list = line.match(/^(- |\d+\. )(.*)$/u)
    if (list) {
      const ordered = !list[1].startsWith("-")
      const items: DocInline[][] = []
      while (index < lines.length) {
        const item = lines[index].match(/^(- |\d+\. )(.*)$/u)
        if (!item) break
        const itemIsOrdered = !item[1].startsWith("-")
        if (itemIsOrdered !== ordered) break
        items.push(parseInline(item[2]))
        index += 1
      }
      blocks.push(emptyBlock({ kind: "list", ordered, items }))
      continue
    }

    const paragraph = [line]
    index += 1
    while (
      index < lines.length &&
      lines[index].trim() !== "" &&
      !startsBlock(lines[index])
    ) {
      paragraph.push(lines[index].trim())
      index += 1
    }
    blocks.push(
      emptyBlock({
        kind: "paragraph",
        inlines: parseInline(paragraph.join(" ")),
      })
    )
  }

  return {
    path,
    metadata,
    documentTitle: title[1].trim(),
    blocks,
  }
}
