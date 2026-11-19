// Search（⌘K）の一覧を作り、絞り込み、並べる。画面には触れない純粋な関数だけを置く。
// 例・レッスン・API・設定を一つの一覧として扱う（DESIGN.md「探す場所は一つ」）。
import type { AnalysisReferenceItem } from "../compiler/types"
import type { DiscoverGroupDefinition } from "../sample-catalog"

export type SearchScope = "examples" | "lessons" | "api" | "settings"
export type SearchFilter = "all" | SearchScope

// 範囲のチップ。ラベルは UI ラベルなので英語のまま出す。
export const searchFilters: readonly Readonly<{
  id: SearchFilter
  label: string
}>[] = [
  { id: "all", label: "All" },
  { id: "examples", label: "Examples" },
  { id: "lessons", label: "Lessons" },
  { id: "api", label: "API" },
  { id: "settings", label: "Settings" },
]

const scopeOrder: readonly SearchScope[] = [
  "examples",
  "lessons",
  "api",
  "settings",
]

const scopeLabel: Readonly<Record<SearchScope, string>> = {
  examples: "Examples",
  lessons: "Lessons",
  api: "API",
  settings: "Settings",
}

export type SearchEntry = Readonly<{
  // 一覧の中で一意な名前（"example:form-todo" など）
  key: string
  scope: SearchScope
  // 範囲を一つに絞ったときの見出し（例のグループ名、レッスンの部の名前など）
  section: string
  title: string
  detail: string
  tag?: string
  // API のシグネチャ。コードの書体で出す
  signature?: string
  searchText: string
}>

export type SearchSection = Readonly<{
  id: string
  title: string
  entries: readonly SearchEntry[]
}>

export function normalizeSearchText(value: string): string {
  return value.normalize("NFKC").toLocaleLowerCase()
}

function searchTextOf(parts: readonly string[]): string {
  return normalizeSearchText(parts.join(" "))
}

export function exampleEntries(
  samples: readonly Readonly<{
    id: string
    title: string
    summary: string
    topics: readonly string[]
  }>[],
  groups: readonly DiscoverGroupDefinition[]
): readonly SearchEntry[] {
  const byId = new Map(samples.map((sample) => [sample.id, sample]))
  return groups.flatMap((group) =>
    group.samples.flatMap((id) => {
      const sample = byId.get(id)
      if (sample === undefined) return []
      return [
        {
          key: `example:${sample.id}`,
          scope: "examples" as const,
          section: group.title,
          title: sample.title,
          detail: sample.summary,
          searchText: searchTextOf([
            sample.title,
            sample.summary,
            group.title,
            sample.id,
            ...sample.topics,
          ]),
        },
      ]
    })
  )
}

type CurriculumShape = Readonly<{
  categories: readonly Readonly<{
    order: number
    title: string
    chapters: readonly Readonly<{
      order: number
      title: string
      lessons: readonly Readonly<{
        id: string
        order: number
        title: string
        summary: string
      }>[]
    }>[]
  }>[]
}>

const byOrder = <T extends Readonly<{ order: number }>>(
  values: readonly T[]
): readonly T[] => [...values].sort((left, right) => left.order - right.order)

export function lessonEntries(
  curriculum: CurriculumShape
): readonly SearchEntry[] {
  return byOrder(curriculum.categories).flatMap((category) =>
    byOrder(category.chapters).flatMap((chapter) =>
      byOrder(chapter.lessons).map((lesson) => ({
        key: `lesson:${lesson.id}`,
        scope: "lessons" as const,
        section: category.title,
        title: lesson.title,
        detail: `${category.title} · ${chapter.title}`,
        searchText: searchTextOf([
          lesson.title,
          lesson.summary,
          chapter.title,
          category.title,
        ]),
      }))
    )
  )
}

export function apiEntries(
  items: readonly AnalysisReferenceItem[]
): readonly SearchEntry[] {
  return items.map((item, index) => ({
    key: `api:${index}:${item.module}:${item.name}`,
    scope: "api" as const,
    section: item.category,
    title: item.name,
    detail: item.description,
    tag: item.module,
    ...(item.signature === undefined ? {} : { signature: item.signature }),
    searchText: searchTextOf([
      item.name,
      item.module,
      item.signature ?? "",
      item.description,
      ...item.constraints,
    ]),
  }))
}

// 何も打っていない All では、件数の多い API を出さない。打ち始めたら出す。
const allLimit = 5
const scopedApiLimit = 80

function rank(entry: SearchEntry, terms: readonly string[]): number {
  const title = normalizeSearchText(entry.title)
  const first = terms[0] ?? ""
  if (title === first) return 0
  if (title.startsWith(first)) return 1
  if (terms.every((term) => title.includes(term))) return 2
  return 3
}

export function searchSections(
  entries: readonly SearchEntry[],
  query: string,
  filter: SearchFilter
): readonly SearchSection[] {
  const terms = normalizeSearchText(query).split(/\s+/u).filter(Boolean)
  const candidates = entries.filter(
    (entry) =>
      (filter === "all" || entry.scope === filter) &&
      (terms.length > 0 || filter !== "all" || entry.scope !== "api") &&
      terms.every((term) => entry.searchText.includes(term))
  )
  // 打った言葉に題名が近いものを先に出す。同じ近さなら、もとの並び（グループ順）を保つ。
  const ranked =
    terms.length === 0
      ? candidates
      : candidates
          .map((entry, index) => ({ entry, index, score: rank(entry, terms) }))
          .sort(
            (left, right) =>
              left.score - right.score || left.index - right.index
          )
          .map(({ entry }) => entry)

  if (filter === "all") {
    return scopeOrder.flatMap((scope) => {
      const scoped = ranked.filter((entry) => entry.scope === scope)
      if (scoped.length === 0) return []
      return [
        {
          id: scope,
          title: scopeLabel[scope],
          entries: scoped.slice(0, allLimit),
        },
      ]
    })
  }

  const limited = filter === "api" ? ranked.slice(0, scopedApiLimit) : ranked
  // 見出しは、その見出しの項目が最初に出た順に並べる。
  const sections = new Map<string, SearchEntry[]>()
  for (const entry of limited) {
    const section = sections.get(entry.section)
    if (section === undefined) sections.set(entry.section, [entry])
    else section.push(entry)
  }
  return [...sections].map(([title, sectionEntries], index) => ({
    id: `${filter}-${index}`,
    title,
    entries: sectionEntries,
  }))
}
