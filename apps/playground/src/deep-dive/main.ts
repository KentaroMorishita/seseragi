import "../ui/design/index.css"
import "../ui/playground.css"
import { requiredElement } from "../ui/elements"
import { renderGuideMarkdown } from "../ui/guide-markdown"
import {
  type DeepDiveArticle,
  deepDiveArticles,
  deepDiveCategories,
  deepDiveChapters,
  findDeepDiveArticle,
} from "./catalog"
import { deepDiveArticleIdFromUrl, deepDiveArticleUrl } from "./route"
import "./deep-dive.css"
import curriculumJson from "../../../../examples/tour/curriculum.json"
import { exampleEntries, lessonEntries } from "../ui/search-index"
import { connectSearchPalette } from "../ui/search-palette"
import { createThemeController } from "../ui/theme"
import { browserStorage, connectThemeButton } from "../ui/theme-button"

const navigation = requiredElement("#deep-dive-navigation", HTMLElement)
const navigationContent = requiredElement(
  "#deep-dive-navigation-content",
  HTMLElement
)
const menuButton = requiredElement("#deep-dive-menu-button", HTMLButtonElement)
const closeButton = requiredElement(
  "#deep-dive-menu-close-button",
  HTMLButtonElement
)
const breadcrumb = requiredElement("#deep-dive-breadcrumb", HTMLElement)
const title = requiredElement("#deep-dive-title", HTMLElement)
const summary = requiredElement("#deep-dive-summary", HTMLElement)
const articleBody = requiredElement("#deep-dive-sections", HTMLElement)
const related = requiredElement("#deep-dive-related", HTMLElement)
const relatedList = requiredElement("#deep-dive-related-list", HTMLUListElement)
const articleIds = deepDiveArticles.map(({ id }) => id)
const mobileNavigationQuery = window.matchMedia("(max-width: 760px)")
const lessonTitles = new Map(
  curriculumJson.categories.flatMap((category) =>
    category.chapters.flatMap((chapter) =>
      chapter.lessons.map((lesson) => [lesson.id, lesson.title] as const)
    )
  )
)
const searchButton = requiredElement("#dd-search-button", HTMLButtonElement)
const themeController = createThemeController({
  storage: browserStorage(),
  root: document.documentElement,
  query: window.matchMedia("(prefers-color-scheme: dark)"),
  eventTarget: window,
})
connectThemeButton(
  requiredElement("#dd-theme-button", HTMLButtonElement),
  themeController
)
// Search は Playground・Tour と同じ。例は Playground で、レッスンは Tour で開く。
const search = connectSearchPalette({
  dialog: requiredElement("#search-dialog", HTMLDialogElement),
  input: requiredElement("#search-input", HTMLInputElement),
  scopes: requiredElement("#search-scopes", HTMLElement),
  results: requiredElement("#search-results", HTMLElement),
  detail: requiredElement("#search-detail", HTMLElement),
  examples: [],
  loadEntries: async () => {
    const { discoverGroups, samples } = await import("../samples")
    return [
      ...exampleEntries(
        samples.filter(({ kind }) => kind !== "lesson"),
        discoverGroups
      ),
      ...lessonEntries(curriculumJson),
    ]
  },
  onExample: (id) => {
    window.location.assign(
      new URL(`../?example=${encodeURIComponent(id)}`, window.location.href)
    )
    return undefined
  },
  onLesson: (id) =>
    window.location.assign(
      new URL(`../tour/?lesson=${encodeURIComponent(id)}`, window.location.href)
    ),
  commands: (["light", "dark", "system"] as const).map((preference) => ({
    id: `theme-${preference}`,
    title: `${preference[0]?.toUpperCase()}${preference.slice(1)} theme`,
    detail: {
      light: "明るい色で表示する",
      dark: "暗い色で表示する",
      system: "OS の設定に合わせる",
    }[preference],
    keywords: "theme テーマ",
    tag: () =>
      themeController.get().preference === preference ? "Current" : undefined,
    run: () => {
      themeController.set(preference)
      return undefined
    },
  })),
  renderApiDetail: () => document.createElement("div"),
})
searchButton.addEventListener("click", () =>
  search.open({ opener: searchButton })
)
document.addEventListener("keydown", (event) => {
  if (!event.metaKey && !event.ctrlKey) return
  if (event.key.toLowerCase() !== "k" || event.shiftKey || event.altKey) return
  event.preventDefault()
  searchButton.click()
})
let currentArticle = findDeepDiveArticle(
  deepDiveArticleIdFromUrl(window.location.href, articleIds)
)

menuButton.addEventListener("click", () => setNavigationOpen(true))
closeButton.addEventListener("click", () => setNavigationOpen(false, true))
window.addEventListener("popstate", () => {
  loadArticle(
    deepDiveArticleIdFromUrl(window.location.href, articleIds),
    "none"
  )
})
navigation.addEventListener("keydown", (event) => {
  if (event.key !== "Escape") return
  event.preventDefault()
  setNavigationOpen(false, true)
})
mobileNavigationQuery.addEventListener("change", () => setNavigationOpen(false))

loadArticle(currentArticle.id, "replace")

function loadArticle(
  articleId: string,
  historyMode: "push" | "replace" | "none"
): void {
  currentArticle = findDeepDiveArticle(articleId)
  if (historyMode !== "none") {
    history[historyMode === "push" ? "pushState" : "replaceState"](
      {},
      "",
      deepDiveArticleUrl(window.location.href, currentArticle.id)
    )
  }
  render()
  setNavigationOpen(false)
  title.focus({ preventScroll: true })
}

function render(): void {
  const category = deepDiveCategories.find(
    ({ id }) => id === currentArticle.categoryId
  )
  const topic = deepDiveChapters.find(
    ({ id }) => id === currentArticle.chapterId
  )
  breadcrumb.textContent = [category?.title, topic?.title]
    .filter((part) => part !== undefined)
    .join(" / ")
  title.textContent = currentArticle.title
  summary.textContent = currentArticle.summary
  articleBody.replaceChildren(
    ...currentArticle.sections.map((section) => {
      const element = document.createElement("section")
      element.id = section.id
      const heading = document.createElement("h2")
      heading.textContent = section.title
      const body = document.createElement("div")
      renderGuideMarkdown(body, section.body)
      element.append(heading, body)
      return element
    })
  )
  renderRelated(currentArticle)
  renderNavigation()
}

function renderRelated(article: DeepDiveArticle): void {
  const tourLinks = article.relatedTourLessons.map((lessonId) => {
    const link = document.createElement("a")
    link.href = `../tour/?lesson=${encodeURIComponent(lessonId)}`
    link.textContent = `Tour: ${lessonTitles.get(lessonId) ?? lessonId}`
    return listItem(link)
  })
  const referenceLinks = article.relatedLinks.map(({ label, href }) => {
    const link = document.createElement("a")
    link.href = href
    link.textContent = label
    link.target = "_blank"
    link.rel = "noreferrer"
    return listItem(link)
  })
  const links = [...tourLinks, ...referenceLinks]
  related.hidden = links.length === 0
  relatedList.replaceChildren(...links)
}

function renderNavigation(): void {
  navigationContent.replaceChildren(
    ...deepDiveCategories.map((category) => {
      const section = document.createElement("section")
      const heading = document.createElement("h2")
      heading.textContent = category.title
      const description = document.createElement("p")
      description.textContent = category.summary
      section.append(heading, description)
      for (const topic of category.chapters) {
        const topicHeading = document.createElement("h3")
        topicHeading.textContent = topic.title
        const list = document.createElement("ul")
        for (const article of topic.articles) {
          const item = document.createElement("li")
          const button = document.createElement("button")
          button.type = "button"
          button.setAttribute(
            "aria-current",
            article.id === currentArticle.id ? "page" : "false"
          )
          button.textContent = article.title
          button.addEventListener("click", () =>
            loadArticle(article.id, "push")
          )
          item.append(button)
          list.append(item)
        }
        section.append(topicHeading, list)
      }
      return section
    })
  )
}

function setNavigationOpen(open: boolean, returnFocus = false): void {
  navigation.dataset.open = String(open)
  menuButton.setAttribute("aria-expanded", String(open))
  if (mobileNavigationQuery.matches) {
    navigation.inert = !open
    navigation.setAttribute("aria-hidden", String(!open))
    // 重ねて開いた一覧の中へフォーカスを移し、Escape で閉じられるようにする
    if (open) closeButton.focus({ preventScroll: true })
  } else {
    navigation.inert = false
    navigation.removeAttribute("aria-hidden")
  }
  if (returnFocus) menuButton.focus({ preventScroll: true })
}

function listItem(child: HTMLElement): HTMLLIElement {
  const item = document.createElement("li")
  item.append(child)
  return item
}
