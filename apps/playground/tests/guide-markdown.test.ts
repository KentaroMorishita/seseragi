import { describe, expect, test } from "bun:test"
import { samples } from "../src/samples"
import { tourLessons } from "../src/tour/curriculum"
import {
  type GuideBlock,
  type GuideInline,
  parseGuideMarkdown,
  renderGuideMarkdown,
  safeGuideLink,
} from "../src/ui/guide-markdown"

describe("guide Markdown", () => {
  test("structures representative sample content", () => {
    const hello = blocksForSample("hello-world")
    expect(inlineKinds(hello)).toContain("code")

    const form = blocksForSample("form-todo")
    expect(form).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ kind: "heading", level: 2 }),
        expect.objectContaining({ kind: "list", ordered: false }),
      ])
    )

    const project = blocksForSample("project-flow-app")
    expect(project).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ kind: "list", ordered: true }),
        expect.objectContaining({ kind: "code-block" }),
      ])
    )
  })

  test("parses every sample and legacy Tour guide through one contract", () => {
    expect(samples).toHaveLength(28)
    for (const sample of samples) {
      const blocks = parseGuideMarkdown(sample.guide)
      expect(blocks.length, sample.id).toBeGreaterThan(0)
      expect(rawBlockMarkers(blocks), sample.id).toEqual([])
    }

    const legacyLessons = tourLessons.filter(
      (lesson) => lesson.format === undefined && lesson.guide.trim() !== ""
    )
    expect(legacyLessons.length).toBeGreaterThan(0)
    for (const lesson of legacyLessons) {
      const blocks = parseGuideMarkdown(lesson.guide)
      expect(blocks.length, lesson.id).toBeGreaterThan(0)
      expect(rawBlockMarkers(blocks), lesson.id).toEqual([])
    }
  })

  test("supports the allowed inline surface and readable line breaks", () => {
    const [paragraph] = parseGuideMarkdown(
      "*emphasis* and **strong** with [docs](https://example.com)  \nnext"
    )
    expect(paragraph?.kind).toBe("paragraph")
    expect(
      paragraph && "children" in paragraph ? paragraph.children : []
    ).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ kind: "emphasis" }),
        expect.objectContaining({ kind: "strong" }),
        expect.objectContaining({ kind: "link", href: "https://example.com" }),
        expect.objectContaining({ kind: "break" }),
      ])
    )
  })

  test("keeps raw HTML inert and refuses dangerous link schemes", () => {
    const blocks = parseGuideMarkdown(
      "<script>globalThis.pwned = true</script> [open](javascript:alert(1))"
    )
    expect(blocks).toHaveLength(1)
    expect(inlineKinds(blocks)).not.toContain("link")
    expect(inlineText(blocks)).toContain("<script>")
    expect(safeGuideLink("javascript:alert(1)")).toBeUndefined()
    expect(safeGuideLink("data:text/html,boom")).toBeUndefined()
    expect(safeGuideLink("//example.com")).toBeUndefined()
    expect(safeGuideLink("mailto:hello@example.com")).toBe(
      "mailto:hello@example.com"
    )
  })

  test("replaces old guide DOM on every render", () => {
    const document = new FakeDocument()
    const target = new FakeElement("div", document)

    renderGuideMarkdown(
      target as unknown as HTMLElement,
      "## Old heading\n\n- stale item"
    )
    expect(target.descendantTags()).toEqual(["h2", "ul", "li"])

    renderGuideMarkdown(target as unknown as HTMLElement, "New paragraph")
    expect(target.descendantTags()).toEqual(["p"])

    renderGuideMarkdown(target as unknown as HTMLElement, "")
    expect(target.descendantTags()).toEqual([])
  })
})

function blocksForSample(id: string): readonly GuideBlock[] {
  const sample = samples.find((candidate) => candidate.id === id)
  if (sample === undefined) throw new Error(`missing sample: ${id}`)
  return parseGuideMarkdown(sample.guide)
}

function inlineKinds(blocks: readonly GuideBlock[]): string[] {
  return blocks.flatMap((block) =>
    "children" in block
      ? nestedKinds(block.children)
      : block.kind === "list"
        ? block.items.flatMap(nestedKinds)
        : []
  )
}

function nestedKinds(nodes: readonly GuideInline[]): string[] {
  return nodes.flatMap((node) => [
    node.kind,
    ...(node.kind === "emphasis" ||
    node.kind === "strong" ||
    node.kind === "link"
      ? nestedKinds(node.children)
      : []),
  ])
}

function inlineText(blocks: readonly GuideBlock[]): string {
  return blocks
    .flatMap((block) =>
      "children" in block
        ? block.children
        : block.kind === "list"
          ? block.items.flat()
          : []
    )
    .flatMap(textOfInline)
    .join("")
}

function textOfInline(node: GuideInline): string[] {
  if (node.kind === "text" || node.kind === "code") return [node.value]
  if (node.kind === "break") return ["\n"]
  return node.children.flatMap(textOfInline)
}

function rawBlockMarkers(blocks: readonly GuideBlock[]): string[] {
  return blocks.flatMap((block) => {
    if (block.kind !== "paragraph") return []
    const text = block.children.flatMap(textOfInline).join("")
    return /^(?:#{2,3}\s|[-+*]\s|\d+[.)]\s|```)/u.test(text) ? [text] : []
  })
}

class FakeElement {
  readonly children: FakeElement[] = []
  readonly dataset: Record<string, string> = {}
  readonly attributes = new Map<string, string>()
  textContent = ""

  constructor(
    readonly tagName: string,
    readonly ownerDocument: FakeDocument
  ) {}

  append(...nodes: FakeElement[]): void {
    for (const node of nodes) {
      if (node.tagName === "#fragment") this.children.push(...node.children)
      else this.children.push(node)
    }
  }

  setAttribute(name: string, value: string): void {
    this.attributes.set(name, value)
  }

  replaceChildren(...nodes: FakeElement[]): void {
    this.children.splice(0)
    this.append(...nodes)
  }

  descendantTags(): string[] {
    return this.children.flatMap((child) => [
      ...(child.tagName.startsWith("#") ? [] : [child.tagName]),
      ...child.descendantTags(),
    ])
  }
}

class FakeDocument {
  createDocumentFragment(): FakeElement {
    return new FakeElement("#fragment", this)
  }

  createElement(tagName: string): FakeElement {
    return new FakeElement(tagName, this)
  }

  createTextNode(value: string): FakeElement {
    const node = new FakeElement("#text", this)
    node.textContent = value
    return node
  }
}
