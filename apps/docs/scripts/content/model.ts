export const locales = ["en", "ja"] as const
export type Locale = (typeof locales)[number]

export const pageKinds = [
  "home",
  "landing",
  "get-started",
  "language",
  "library",
  "application",
  "interop",
  "tooling",
  "internals",
] as const
export type PageKind = (typeof pageKinds)[number]

export const availabilityStates = [
  "available",
  "contract-only",
  "internal",
] as const
export type Availability = (typeof availabilityStates)[number]

export type PageMetadata = {
  schema: 1
  id: string
  locale: Locale
  route: string
  kind: PageKind
  title: string
  summary: string
  availability: Availability
  spec: string[]
  examples: string[]
  prerequisites: string[]
  related: string[]
  next: string
  referenceIdentities: string[]
}

export type DocInline = {
  kind: "text" | "code" | "emphasis" | "strong" | "link"
  text: string
  href: string
}

export type ParsedDocBlock = {
  kind:
    | "paragraph"
    | "heading"
    | "list"
    | "code-block"
    | "example"
    | "from-typescript"
    | "design-rationale"
    | "common-mistake"
    | "warning"
    | "availability"
    | "api-reference"
  inlines: DocInline[]
  level: number
  ordered: boolean
  items: DocInline[][]
  title: string
  text: string
  source: string
  language: string
  reference: string
  referenceName: string
  referenceKind: string
  referenceNamespace: string
  referenceSignature: string
  referenceDescription: string
}

export type ParsedPage = {
  path: string
  metadata: PageMetadata
  documentTitle: string
  blocks: ParsedDocBlock[]
}

export type SourceProvenance = {
  path: string
  section: string
  sha256: string
}

export type PreparedDocBlock = ParsedDocBlock & {
  sha256: string
}

export type PreparedPage = Omit<ParsedPage, "path" | "blocks"> & {
  sourcePath: string
  spec: SourceProvenance[]
  blocks: PreparedDocBlock[]
}

export type DocCorpus = {
  schema: 1
  defaultLocale: Locale
  locales: Locale[]
  pages: PreparedPage[]
}
