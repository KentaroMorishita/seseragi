import "../styles.css"
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
const breadcrumb = requiredElement("#deep-dive-breadcrumb", HTMLElement)
const title = requiredElement("#deep-dive-title", HTMLElement)
const summary = requiredElement("#deep-dive-summary", HTMLElement)
const articleBody = requiredElement("#deep-dive-sections", HTMLElement)
const related = requiredElement("#deep-dive-related", HTMLElement)
const relatedList = requiredElement("#deep-dive-related-list", HTMLUListElement)
const articleIds = deepDiveArticles.map(({ id }) => id)
const mobileNavigationQuery = window.matchMedia("(max-width: 760px)")
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
    link.textContent = `Tour: ${lessonId}`
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
