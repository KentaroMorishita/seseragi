import "../styles.css"
import { highlightSeseragi } from "../editor/seseragi-language"
import { requiredElement } from "../ui/elements"
import { renderGuideMarkdown } from "../ui/guide-markdown"
import {
  type DeepDiveArticle,
  deepDiveArticles,
  deepDiveCategories,
  deepDiveChapters,
  findDeepDiveArticle,
} from "./catalog"
import { buildDeepDiveNavigation } from "./navigation"
import {
  completeDeepDiveArticle,
  type DeepDiveProgress,
  loadDeepDiveProgress,
  saveDeepDiveProgress,
  visitDeepDiveArticle,
} from "./progress"
import { deepDiveArticleIdFromUrl, deepDiveArticleUrl } from "./route"
import "./styles.css"

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
const progressElement = requiredElement(
  "#deep-dive-progress",
  HTMLProgressElement
)
const progressLabel = requiredElement("#deep-dive-progress-label", HTMLElement)
const breadcrumb = requiredElement("#deep-dive-breadcrumb", HTMLElement)
const title = requiredElement("#deep-dive-title", HTMLElement)
const summary = requiredElement("#deep-dive-summary", HTMLElement)
const prerequisites = requiredElement("#deep-dive-prerequisites", HTMLElement)
const prerequisiteList = requiredElement(
  "#deep-dive-prerequisite-list",
  HTMLUListElement
)
const articleBody = requiredElement("#deep-dive-sections", HTMLElement)
const sourceCode = requiredElement("#deep-dive-source", HTMLElement)
const expectedOutput = requiredElement(
  "#deep-dive-expected-output",
  HTMLElement
)
const diagnosticSource = requiredElement(
  "#deep-dive-diagnostic-source",
  HTMLElement
)
const diagnosticOutput = requiredElement(
  "#deep-dive-diagnostic-output",
  HTMLElement
)
const recapList = requiredElement("#deep-dive-recap-list", HTMLUListElement)
const completeButton = requiredElement(
  "#deep-dive-complete-button",
  HTMLButtonElement
)
const previousButton = requiredElement(
  "#deep-dive-previous-button",
  HTMLButtonElement
)
const nextButton = requiredElement("#deep-dive-next-button", HTMLButtonElement)
const articleIds = deepDiveArticles.map(({ id }) => id)
const mobileNavigationQuery = window.matchMedia("(max-width: 760px)")
const requestedArticleParameter = new URL(
  window.location.href
).searchParams.get("article")
const requestedArticleId =
  requestedArticleParameter !== null &&
  articleIds.includes(requestedArticleParameter)
    ? requestedArticleParameter
    : null
let progress: DeepDiveProgress = loadDeepDiveProgress(
  localStorage,
  articleIds,
  requestedArticleId
)
let currentArticle = findDeepDiveArticle(progress.currentArticleId)

menuButton.addEventListener("click", () => setNavigationOpen(true))
closeButton.addEventListener("click", () => setNavigationOpen(false, true))
completeButton.addEventListener("click", () => {
  progress = completeDeepDiveArticle(progress, currentArticle.id)
  persistProgress()
  render()
})
previousButton.addEventListener("click", () => moveArticle(-1))
nextButton.addEventListener("click", () => moveArticle(1))
window.addEventListener("popstate", () => {
  const articleId = deepDiveArticleIdFromUrl(window.location.href, articleIds)
  loadArticle(articleId, "none")
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
  progress = visitDeepDiveArticle(progress, currentArticle.id)
  persistProgress()
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
  const chapter = deepDiveChapters.find(
    ({ id }) => id === currentArticle.chapterId
  )
  const model = buildDeepDiveNavigation(
    deepDiveCategories,
    deepDiveArticles,
    currentArticle.id,
    progress.completedArticleIds
  )
  breadcrumb.textContent = [category?.title, chapter?.title]
    .filter((part) => part !== undefined)
    .join(" / ")
  title.textContent = currentArticle.title
  summary.textContent = currentArticle.summary
  progressElement.max = model.progress.total
  progressElement.value = model.progress.completed
  progressLabel.textContent = `${model.progress.completed} / ${model.progress.total} completed`
  completeButton.disabled = progress.completedArticleIds.includes(
    currentArticle.id
  )
  completeButton.textContent = completeButton.disabled
    ? "✓ 完了済み"
    : "この記事を完了にする"
  renderPrerequisites(currentArticle)
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
  renderSeseragiSource(sourceCode, currentArticle.source)
  expectedOutput.textContent = currentArticle.expectedOutput
  renderSeseragiSource(diagnosticSource, currentArticle.diagnosticSource)
  diagnosticOutput.textContent = currentArticle.diagnosticOutput
  recapList.replaceChildren(
    ...currentArticle.recap.map((item) => {
      const element = document.createElement("li")
      element.textContent = item
      return element
    })
  )
  renderNavigation(model.categories)
  const index = deepDiveArticles.findIndex(({ id }) => id === currentArticle.id)
  renderNeighbor(previousButton, deepDiveArticles[index - 1], "← 前の記事")
  renderNeighbor(nextButton, deepDiveArticles[index + 1], "次の記事 →")
}

function renderSeseragiSource(target: HTMLElement, source: string): void {
  target.replaceChildren(
    ...highlightSeseragi(source).map(({ text, classes }) => {
      const part = document.createElement("span")
      part.textContent = text
      if (classes !== "") part.className = classes
      return part
    })
  )
}

function renderPrerequisites(article: DeepDiveArticle): void {
  const deepDivePrerequisites = article.prerequisites.flatMap((id) => {
    const prerequisite = deepDiveArticles.find(
      (candidate) => candidate.id === id
    )
    if (prerequisite === undefined) return []
    const link = document.createElement("a")
    link.href = `?article=${encodeURIComponent(prerequisite.id)}`
    link.textContent = `Deep Dive: ${prerequisite.title}`
    link.addEventListener("click", (event) => {
      event.preventDefault()
      loadArticle(prerequisite.id, "push")
    })
    return [listItem(link)]
  })
  const tourPrerequisites = article.tourPrerequisites.map((id) => {
    const link = document.createElement("a")
    link.href = `../tour/?lesson=${encodeURIComponent(id)}`
    link.textContent = `Tour lesson: ${id}`
    return listItem(link)
  })
  const items = [...tourPrerequisites, ...deepDivePrerequisites]
  prerequisites.hidden = items.length === 0
  prerequisiteList.replaceChildren(...items)
}

function renderNavigation(
  categories: ReturnType<typeof buildDeepDiveNavigation>["categories"]
): void {
  navigationContent.replaceChildren(
    ...categories.map((category) => {
      const section = document.createElement("section")
      const heading = document.createElement("h2")
      heading.textContent = category.title
      const description = document.createElement("p")
      description.textContent = category.summary
      section.append(heading, description)
      for (const chapter of category.chapters) {
        const chapterHeading = document.createElement("h3")
        chapterHeading.textContent = chapter.title
        const list = document.createElement("ol")
        for (const article of chapter.articles) {
          const item = document.createElement("li")
          const button = document.createElement("button")
          button.type = "button"
          button.dataset.state = article.state
          button.setAttribute(
            "aria-current",
            article.state === "current" ? "page" : "false"
          )
          button.textContent = `${String(article.position).padStart(2, "0")} ${article.title}`
          button.addEventListener("click", () =>
            loadArticle(article.id, "push")
          )
          item.append(button)
          list.append(item)
        }
        section.append(chapterHeading, list)
      }
      return section
    })
  )
}

function renderNeighbor(
  button: HTMLButtonElement,
  article: DeepDiveArticle | undefined,
  direction: string
): void {
  button.disabled = article === undefined
  button.hidden = article === undefined
  button.textContent =
    article === undefined ? "" : `${direction} · ${article.title}`
}

function moveArticle(offset: number): void {
  const index = deepDiveArticles.findIndex(({ id }) => id === currentArticle.id)
  const article = deepDiveArticles[index + offset]
  if (article !== undefined) loadArticle(article.id, "push")
}

function setNavigationOpen(open: boolean, returnFocus = false): void {
  navigation.dataset.open = String(open)
  menuButton.setAttribute("aria-expanded", String(open))
  if (mobileNavigationQuery.matches) {
    navigation.inert = !open
    navigation.setAttribute("aria-hidden", String(!open))
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

function persistProgress(): void {
  try {
    saveDeepDiveProgress(localStorage, progress)
  } catch {
    // Deep Dive remains usable when storage is unavailable.
  }
}
