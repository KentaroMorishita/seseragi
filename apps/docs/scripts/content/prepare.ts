import assert from "node:assert/strict"
import { createHash } from "node:crypto"
import { readFileSync, realpathSync } from "node:fs"
import { dirname, join, relative, resolve } from "node:path"
import {
  discoverAuthoringFiles,
  readContentConfig,
  readGlossaries,
} from "./discover"
import type { DocCorpus, PreparedDocBlock, SourceProvenance } from "./model"
import { parseAuthoringFile } from "./parse"
import { validateGlossaries, validatePages } from "./validate"

const digest = (value: string) =>
  createHash("sha256").update(value).digest("hex")

function safeRepositorySource(repositoryRoot: string, path: string): string {
  assert.match(path, /^[a-zA-Z0-9_./-]+$/u, `Unsafe repository path: ${path}`)
  assert.ok(!path.split("/").includes(".."), `Unsafe repository path: ${path}`)
  const absolute = realpathSync(resolve(repositoryRoot, path))
  assert.ok(
    !relative(repositoryRoot, absolute).startsWith(".."),
    `Repository source escapes root: ${path}`
  )
  return absolute
}

function prepareSpec(
  repositoryRoot: string,
  reference: string
): SourceProvenance {
  const match = reference.match(
    /^(docs\/spec\/[^#]+\.md)#(\d+\.\d+(?:\.\d+)*)$/u
  )
  assert.ok(match, `Invalid specification reference: ${reference}`)
  const absolute = safeRepositorySource(repositoryRoot, match[1])
  const source = readFileSync(absolute, "utf8")
  assert.ok(
    source
      .split("\n")
      .some((line) =>
        new RegExp(`^#{2,3} ${match[2].replaceAll(".", "\\.")} `).test(line)
      ),
    `Unknown specification section: ${reference}`
  )
  return { path: match[1], section: match[2], sha256: digest(source) }
}

function prepareBlock(
  repositoryRoot: string,
  block: PreparedDocBlock,
  references: Map<
    string,
    {
      name: string
      kind: string
      namespace: string
      signature: string
      description: string
    }
  >
): PreparedDocBlock {
  if (block.kind === "api-reference") {
    const reference = references.get(block.reference)
    assert.ok(
      reference,
      `Unknown compiler Reference identity: ${block.reference}`
    )
    return {
      ...block,
      referenceName: reference.name,
      referenceKind: reference.kind,
      referenceNamespace: reference.namespace,
      referenceSignature: reference.signature,
      referenceDescription: reference.description,
      sha256: digest(JSON.stringify(reference)),
    }
  }
  if (block.kind !== "example") return { ...block, sha256: "" }
  assert.match(
    block.source,
    /^examples\/spec\/[a-zA-Z0-9_./-]+\.ssrg$/u,
    `Invalid canonical example path: ${block.source}`
  )
  const absolute = safeRepositorySource(repositoryRoot, block.source)
  const source = readFileSync(absolute, "utf8")
  return {
    ...block,
    text: source,
    language: "seseragi",
    sha256: digest(source),
  }
}

export function prepareContent(options: {
  repositoryRoot: string
  contentRoot: string
}): DocCorpus {
  const config = readContentConfig(options.contentRoot)
  const glossaries = readGlossaries(options.contentRoot)
  validateGlossaries(glossaries)
  const pagesRoot = join(options.contentRoot, "pages")
  const pages = discoverAuthoringFiles(options.contentRoot).map((path) =>
    parseAuthoringFile(readFileSync(path, "utf8"), path)
  )
  validatePages(pages, pagesRoot)
  const artifact = JSON.parse(
    readFileSync(
      join(
        options.repositoryRoot,
        "examples/spec/artifacts/stdlib-schema-1/reference/module.json"
      ),
      "utf8"
    )
  ) as { modules: Array<{ items: Array<Record<string, unknown>> }> }
  const references = new Map(
    artifact.modules.flatMap(({ items }) =>
      items.map((item) => {
        const identity = item.identity
        assert.equal(typeof identity, "string")
        for (const key of [
          "name",
          "kind",
          "namespace",
          "signature",
          "description",
        ])
          assert.equal(
            typeof item[key],
            "string",
            `Invalid Reference ${identity} ${key}`
          )
        return [
          identity as string,
          {
            name: item.name as string,
            kind: item.kind as string,
            namespace: item.namespace as string,
            signature: item.signature as string,
            description: item.description as string,
          },
        ] as const
      })
    )
  )

  return {
    schema: 1,
    defaultLocale: config.defaultLocale,
    locales: config.locales,
    pages: pages.map((page) => ({
      sourcePath: relative(options.repositoryRoot, page.path),
      metadata: page.metadata,
      documentTitle: page.documentTitle,
      spec: page.metadata.spec.map((reference) =>
        prepareSpec(options.repositoryRoot, reference)
      ),
      blocks: page.blocks.map((block) =>
        prepareBlock(
          options.repositoryRoot,
          { ...block, sha256: "" },
          references
        )
      ),
    })),
  }
}

export function toLegacyPages(corpus: DocCorpus, locale: "en" | "ja") {
  type LegacyBlock =
    | { kind: "paragraph" | "heading" | "terminal"; text: string }
    | { kind: "sample"; source: string }
  return corpus.pages
    .filter(({ metadata }) => metadata.locale === locale)
    .map((page) => ({
      id: page.metadata.id.replaceAll(".", "-"),
      route: page.metadata.route,
      title: page.metadata.title,
      summary: page.metadata.summary,
      blocks: page.blocks.flatMap<LegacyBlock>((block) => {
        const inlineText = block.inlines.map(({ text }) => text).join("")
        switch (block.kind) {
          case "paragraph":
            return [{ kind: "paragraph" as const, text: inlineText }]
          case "heading":
            return [{ kind: "heading" as const, text: inlineText }]
          case "list":
            return [
              {
                kind: "paragraph" as const,
                text: block.items
                  .map((item) => `- ${item.map(({ text }) => text).join("")}`)
                  .join("\n"),
              },
            ]
          case "example":
            return [{ kind: "sample" as const, source: block.source }]
          case "code-block":
            return [{ kind: "terminal" as const, text: block.text }]
          case "api-reference":
            return [{ kind: "paragraph" as const, text: block.reference }]
          default:
            return [
              ...(block.title
                ? [{ kind: "heading" as const, text: block.title }]
                : []),
              {
                kind: "paragraph" as const,
                text: inlineText || block.text,
              },
            ]
        }
      }),
    }))
}

export function repositoryContentRoot(importMetaDir: string): {
  repositoryRoot: string
  contentRoot: string
} {
  const contentRoot = resolve(importMetaDir, "../../content")
  return {
    repositoryRoot: resolve(dirname(contentRoot), "../.."),
    contentRoot,
  }
}
