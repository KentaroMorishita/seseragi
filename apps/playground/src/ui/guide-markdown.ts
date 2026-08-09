export type GuideInline =
  | Readonly<{ kind: "text"; value: string }>
  | Readonly<{ kind: "code"; value: string }>
  | Readonly<{ kind: "emphasis"; children: readonly GuideInline[] }>
  | Readonly<{ kind: "strong"; children: readonly GuideInline[] }>
  | Readonly<{
      kind: "link"
      href: string
      children: readonly GuideInline[]
    }>
  | Readonly<{ kind: "break" }>

export type GuideBlock =
  | Readonly<{ kind: "paragraph"; children: readonly GuideInline[] }>
  | Readonly<{
      kind: "heading"
      level: 2 | 3
      children: readonly GuideInline[]
    }>
  | Readonly<{
      kind: "list"
      ordered: boolean
      items: readonly (readonly GuideInline[])[]
    }>
  | Readonly<{
      kind: "code-block"
      value: string
      language?: string
    }>

type ListMatch = Readonly<{ ordered: boolean; value: string }>

export function parseGuideMarkdown(markdown: string): readonly GuideBlock[] {
  const normalized = markdown.replace(/\r\n?/gu, "\n").trim()
  if (normalized === "") return []

  const lines = normalized.split("\n")
  const blocks: GuideBlock[] = []
  let index = 0

  while (index < lines.length) {
    const line = lines[index] ?? ""
    if (line.trim() === "") {
      index += 1
      continue
    }

    const fence = line.match(/^ {0,3}(`{3,}|~{3,})\s*([^\s`]*)\s*$/u)
    if (fence !== null) {
      const marker = fence[1] ?? "```"
      const code: string[] = []
      index += 1
      while (index < lines.length && !closesFence(lines[index] ?? "", marker)) {
        code.push(lines[index] ?? "")
        index += 1
      }
      if (index < lines.length) index += 1
      const language = fence[2]
      blocks.push({
        kind: "code-block",
        value: code.join("\n"),
        ...(language === undefined || language === "" ? {} : { language }),
      })
      continue
    }

    const heading = line.match(/^ {0,3}(#{2,3})\s+(.+?)\s*#*\s*$/u)
    if (heading !== null) {
      blocks.push({
        kind: "heading",
        level: heading[1]?.length === 3 ? 3 : 2,
        children: parseGuideInline(heading[2] ?? ""),
      })
      index += 1
      continue
    }

    const firstItem = matchListItem(line)
    if (firstItem !== undefined) {
      const items: GuideInline[][] = []
      let value = firstItem.value
      const ordered = firstItem.ordered
      index += 1

      while (index <= lines.length) {
        const next = lines[index]
        if (next === undefined || next.trim() === "") {
          items.push([...parseGuideInline(value)])
          if (next !== undefined) index += 1
          break
        }

        const item = matchListItem(next)
        if (item !== undefined) {
          if (item.ordered !== ordered) {
            items.push([...parseGuideInline(value)])
            break
          }
          items.push([...parseGuideInline(value)])
          value = item.value
          index += 1
          continue
        }

        if (startsBlock(next)) {
          items.push([...parseGuideInline(value)])
          break
        }

        value += `\n${next.trimStart()}`
        index += 1
      }

      blocks.push({ kind: "list", ordered, items })
      continue
    }

    const paragraph = [line.trimStart()]
    index += 1
    while (index < lines.length) {
      const next = lines[index] ?? ""
      if (next.trim() === "" || startsBlock(next)) break
      paragraph.push(next.trimStart())
      index += 1
    }
    blocks.push({
      kind: "paragraph",
      children: parseGuideInline(paragraph.join("\n")),
    })
  }

  return blocks
}

export function renderGuideMarkdown(
  target: HTMLElement,
  markdown: string
): void {
  const document = target.ownerDocument
  const fragment = document.createDocumentFragment()

  for (const block of parseGuideMarkdown(markdown)) {
    if (block.kind === "code-block") {
      const pre = document.createElement("pre")
      const code = document.createElement("code")
      code.textContent = block.value
      if (block.language !== undefined) {
        code.dataset.language = block.language
      }
      pre.append(code)
      fragment.append(pre)
      continue
    }

    if (block.kind === "list") {
      const list = document.createElement(block.ordered ? "ol" : "ul")
      for (const item of block.items) {
        const listItem = document.createElement("li")
        appendInline(document, listItem, item)
        list.append(listItem)
      }
      fragment.append(list)
      continue
    }

    const element = document.createElement(
      block.kind === "heading" ? `h${block.level}` : "p"
    )
    appendInline(document, element, block.children)
    fragment.append(element)
  }

  target.replaceChildren(fragment)
}

export function renderGuideInline(target: HTMLElement, source: string): void {
  const document = target.ownerDocument
  const fragment = document.createDocumentFragment()
  appendInline(document, fragment, parseGuideInline(source))
  target.replaceChildren(fragment)
}

export function guideInlineSourceProblem(source: string): string | undefined {
  if (
    source
      .replace(/\r\n?/gu, "\n")
      .split("\n")
      .some((line) =>
        /^ {0,3}(?:#{1,6}\s|[-+*]\s|\d+[.)]\s|`{3,}|~{3,}|>\s)/u.test(line)
      )
  ) {
    return "block Markdown is not allowed"
  }
  if (/!\[[^\]]*\]\([^\n)]*\)/u.test(source)) {
    return "images are not allowed"
  }

  for (const match of source.matchAll(/\[[^\]\n]+\]\(([^\n)]*)\)/gu)) {
    if (safeGuideLink(match[1] ?? "") === undefined) {
      return "link uses an unsafe scheme"
    }
  }

  let index = 0
  while (index < source.length) {
    if (source[index] === "\\") {
      index += 2
      continue
    }
    if (source[index] !== "`") {
      index += 1
      continue
    }
    const size = delimiterSize(source, index, "`")
    const close = source.indexOf("`".repeat(size), index + size)
    if (close === -1) return "inline code delimiter is not closed"
    index = close + size
  }
  return undefined
}

export function safeGuideLink(value: string): string | undefined {
  const href = value.trim()
  if (
    href === "" ||
    [...href].some((character) => {
      const code = character.codePointAt(0) ?? 0
      return code <= 31 || code === 127
    })
  ) {
    return undefined
  }
  if (href.startsWith("//")) return undefined

  const scheme = href.match(/^([a-z][a-z\d+.-]*):/iu)?.[1]?.toLowerCase()
  if (scheme !== undefined && !["http", "https", "mailto"].includes(scheme)) {
    return undefined
  }
  return href
}

export function parseGuideInline(source: string): readonly GuideInline[] {
  const nodes: GuideInline[] = []
  let text = ""
  let index = 0

  const flushText = (): void => {
    if (text === "") return
    const previous = nodes.at(-1)
    if (previous?.kind === "text") {
      nodes[nodes.length - 1] = {
        kind: "text",
        value: previous.value + text,
      }
    } else {
      nodes.push({ kind: "text", value: text })
    }
    text = ""
  }

  while (index < source.length) {
    if (source[index] === "\n") {
      const hardBreak = text.endsWith("  ") || text.endsWith("\\")
      text = hardBreak
        ? text.slice(0, text.length - (text.endsWith("  ") ? 2 : 1))
        : `${text} `
      flushText()
      if (hardBreak) nodes.push({ kind: "break" })
      index += 1
      continue
    }

    if (source[index] === "\\" && index + 1 < source.length) {
      text += source[index + 1]
      index += 2
      continue
    }

    if (source[index] === "`") {
      const size = delimiterSize(source, index, "`")
      const delimiter = "`".repeat(size)
      const close = source.indexOf(delimiter, index + size)
      if (close !== -1) {
        flushText()
        const value = source
          .slice(index + size, close)
          .replace(/^ (.*) $/su, "$1")
        nodes.push({ kind: "code", value })
        index = close + size
        continue
      }
    }

    if (source[index] === "[") {
      const labelEnd = source.indexOf("](", index + 1)
      const hrefEnd = labelEnd === -1 ? -1 : source.indexOf(")", labelEnd + 2)
      if (labelEnd !== -1 && hrefEnd !== -1) {
        const label = source.slice(index + 1, labelEnd)
        const href = safeGuideLink(source.slice(labelEnd + 2, hrefEnd))
        flushText()
        if (href === undefined) {
          nodes.push({ kind: "text", value: label })
        } else {
          nodes.push({
            kind: "link",
            href,
            children: parseGuideInline(label),
          })
        }
        index = hrefEnd + 1
        continue
      }
    }

    const strongDelimiter = source.startsWith("**", index)
      ? "**"
      : source.startsWith("__", index)
        ? "__"
        : undefined
    if (strongDelimiter !== undefined) {
      const close = source.indexOf(strongDelimiter, index + 2)
      if (close > index + 2) {
        flushText()
        nodes.push({
          kind: "strong",
          children: parseGuideInline(source.slice(index + 2, close)),
        })
        index = close + 2
        continue
      }
    }

    const emphasisDelimiter = source[index]
    if (emphasisDelimiter === "*" || emphasisDelimiter === "_") {
      const close = source.indexOf(emphasisDelimiter, index + 1)
      if (close > index + 1) {
        flushText()
        nodes.push({
          kind: "emphasis",
          children: parseGuideInline(source.slice(index + 1, close)),
        })
        index = close + 1
        continue
      }
    }

    text += source[index]
    index += 1
  }

  flushText()
  return nodes
}

function appendInline(
  document: Document,
  parent: HTMLElement | DocumentFragment,
  nodes: readonly GuideInline[]
): void {
  for (const node of nodes) {
    if (node.kind === "text") {
      parent.append(document.createTextNode(node.value))
      continue
    }
    if (node.kind === "break") {
      parent.append(document.createElement("br"))
      continue
    }

    const element = document.createElement(
      node.kind === "code"
        ? "code"
        : node.kind === "strong"
          ? "strong"
          : node.kind === "emphasis"
            ? "em"
            : "a"
    )
    if (node.kind === "code") {
      element.textContent = node.value
    } else {
      if (node.kind === "link") {
        element.setAttribute("href", node.href)
        if (/^https?:/iu.test(node.href)) {
          element.setAttribute("target", "_blank")
          element.setAttribute("rel", "noopener noreferrer")
        }
      }
      appendInline(document, element, node.children)
    }
    parent.append(element)
  }
}

function matchListItem(line: string): ListMatch | undefined {
  const unordered = line.match(/^ {0,3}[-+*]\s+(.+)$/u)
  if (unordered !== null) {
    return { ordered: false, value: unordered[1] ?? "" }
  }
  const ordered = line.match(/^ {0,3}\d+[.)]\s+(.+)$/u)
  return ordered === null
    ? undefined
    : { ordered: true, value: ordered[1] ?? "" }
}

function startsBlock(line: string): boolean {
  return (
    /^ {0,3}(?:#{2,3})\s+/u.test(line) ||
    /^ {0,3}(?:`{3,}|~{3,})/u.test(line) ||
    matchListItem(line) !== undefined
  )
}

function closesFence(line: string, marker: string): boolean {
  const character = marker[0] ?? "`"
  return new RegExp(
    `^ {0,3}${character === "`" ? "`" : "~"}{${marker.length},}\\s*$`,
    "u"
  ).test(line)
}

function delimiterSize(source: string, start: number, value: string): number {
  let end = start
  while (source[end] === value) end += 1
  return end - start
}
