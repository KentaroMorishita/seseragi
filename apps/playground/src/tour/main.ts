import { setDiagnostics } from "@codemirror/lint"
import { analysisHoverAt } from "../analysis/hover"
import {
  createLiveAnalysis,
  type LiveAnalysisController,
} from "../analysis/live-analysis"
import type { AnalysisDocument, Diagnostic } from "../compiler/types"
import {
  analyzeSingleFile,
  compileSingleFile,
  formatSingleFile,
} from "../compiler/wasm-driver"
import { renderDiagnosticCards } from "../diagnostics/diagnostic-cards"
import { toEditorDiagnostics } from "../diagnostics/editor-diagnostics"
import { utf8RangeToUtf16 } from "../diagnostics/source-range"
import { createEditor, replaceEditorSource } from "../editor/create-editor"
import { createPreviewDocument } from "../preview-document"
import {
  type BrowserExecution,
  startGeneratedModule,
} from "../runtime/browser-execution"
import "../styles.css"
import { requiredElement } from "../ui/elements"
import { renderGuideInline, renderGuideMarkdown } from "../ui/guide-markdown"
import { connectPreviewFullscreen } from "../ui/preview-fullscreen"
import type {
  TourInlineRichText,
  TourLessonFormat,
  TourSourceRange,
  TourWalkthroughStep,
} from "./content"
import {
  findTourLesson,
  tourCategories,
  tourChapters,
  tourLessons,
} from "./curriculum"
import {
  buildTourNavigationModel,
  type TourLessonNeighbor,
  type TourProgressSummary,
  tourLessonNeighbors,
} from "./navigation"
import {
  completeTourLesson,
  loadTourProgress,
  saveTourProgress,
  type TourProgress,
  visitTourLesson,
} from "./progress"
import "./styles.css"

const chapterHost = requiredElement("#tour-chapters", HTMLElement)
const menuButton = requiredElement("#tour-menu-button", HTMLButtonElement)
const menuCloseButton = requiredElement(
  "#tour-menu-close-button",
  HTMLButtonElement
)
const navigation = requiredElement("#tour-navigation", HTMLElement)
const topbar = requiredElement(".tour-topbar", HTMLElement)
const lesson = requiredElement(".tour-lesson", HTMLElement)
const lab = requiredElement(".tour-lab", HTMLElement)
const stepLabel = requiredElement("#tour-step-label", HTMLElement)
const progressBar = requiredElement("#tour-progress", HTMLProgressElement)
const progressLabel = requiredElement("#tour-progress-label", HTMLElement)
const chapterLabel = requiredElement("#tour-chapter-label", HTMLElement)
const lessonTitle = requiredElement("#tour-lesson-title", HTMLElement)
const lessonSummary = requiredElement("#tour-lesson-summary", HTMLElement)
const goal = requiredElement("#tour-goal", HTMLElement)
const focusList = requiredElement("#tour-focus-list", HTMLUListElement)
const prerequisiteSection = requiredElement(
  "#tour-prerequisite-section",
  HTMLElement
)
const prerequisiteCopy = requiredElement("#tour-prerequisite-copy", HTMLElement)
const prerequisiteList = requiredElement(
  "#tour-prerequisite-list",
  HTMLUListElement
)
const runSection = requiredElement("#tour-run-section", HTMLElement)
const runCopy = requiredElement("#tour-run-copy", HTMLElement)
const expectedOutput = requiredElement("#tour-expected-output", HTMLElement)
const guideSection = requiredElement("#tour-guide-section", HTMLElement)
const lessonGuide = requiredElement("#tour-guide", HTMLElement)
const walkthroughSection = requiredElement(
  "#tour-walkthrough-section",
  HTMLElement
)
const walkthrough = requiredElement("#tour-walkthrough", HTMLElement)
const introducedSection = requiredElement(
  "#tour-introduced-section",
  HTMLElement
)
const introduced = requiredElement("#tour-introduced", HTMLElement)
const trySection = requiredElement("#tour-try-section", HTMLElement)
const challenge = requiredElement("#tour-challenge", HTMLElement)
const topicList = requiredElement("#tour-topic-list", HTMLUListElement)
const exerciseSection = requiredElement("#tour-exercise-section", HTMLElement)
const exerciseCopy = requiredElement("#tour-exercise-copy", HTMLElement)
const exerciseButton = requiredElement(
  "#tour-exercise-button",
  HTMLButtonElement
)
const exerciseOutput = requiredElement("#tour-exercise-output", HTMLElement)
const diagnosticSection = requiredElement(
  "#tour-diagnostic-section",
  HTMLElement
)
const diagnosticHeading = requiredElement(
  "#tour-diagnostic-heading",
  HTMLElement
)
const diagnosticCopy = requiredElement("#tour-diagnostic-copy", HTMLElement)
const diagnosticButton = requiredElement(
  "#tour-diagnostic-button",
  HTMLButtonElement
)
const diagnosticOutput = requiredElement("#tour-diagnostic-output", HTMLElement)
const recapSection = requiredElement("#tour-recap-section", HTMLElement)
const recapList = requiredElement("#tour-recap-list", HTMLUListElement)
const nextSection = requiredElement("#tour-next-section", HTMLElement)
const nextCopy = requiredElement("#tour-next-copy", HTMLElement)
const notesList = requiredElement("#tour-notes-list", HTMLUListElement)
const previousButton = requiredElement(
  "#tour-previous-button",
  HTMLButtonElement
)
const nextButton = requiredElement("#tour-next-button", HTMLButtonElement)
const runButton = requiredElement("#tour-run-button", HTMLButtonElement)
const resetButton = requiredElement("#tour-reset-button", HTMLButtonElement)
const formatButton = requiredElement("#tour-format-button", HTMLButtonElement)
const editorHost = requiredElement("#tour-editor", HTMLDivElement)
const statusDot = requiredElement("#tour-status-dot", HTMLElement)
const statusText = requiredElement("#tour-status-text", HTMLElement)
const inputSection = requiredElement("#tour-input-section", HTMLElement)
const stdinInput = requiredElement("#tour-stdin-input", HTMLTextAreaElement)
const outputSection = requiredElement("#tour-output-section", HTMLElement)
const output = requiredElement("#tour-output", HTMLElement)
const htmlPreview = requiredElement("#tour-html-preview", HTMLIFrameElement)
const showTextButton = requiredElement(
  "#tour-show-text-button",
  HTMLButtonElement
)
const showPreviewButton = requiredElement(
  "#tour-show-preview-button",
  HTMLButtonElement
)
const fullscreenButton = requiredElement(
  "#tour-fullscreen-button",
  HTMLButtonElement
)
const collapsibleNavigationQuery = window.matchMedia("(max-width: 1180px)")
const mobileNavigationQuery = window.matchMedia(
  "(max-width: 760px), (max-width: 960px) and (max-height: 520px)"
)

const lessonIds = tourLessons.map(({ id }) => id)
const requestedLesson = new URL(window.location.href).searchParams.get("lesson")
let progress: TourProgress = loadTourProgress(
  localStorage,
  lessonIds,
  requestedLesson
)
let currentLesson = findTourLesson(progress.currentLessonId)
let source = currentLesson.source
let outputMode: "text" | "html" = currentLesson.outputMode
let latestAnalysis: AnalysisDocument | undefined
let activeExecution: BrowserExecution | undefined
let runRevision = 0
let htmlPreviewUrl: string | undefined
let navigationBackgroundScrollTop = 0
const expandedCategoryIds = new Set<string>([currentLesson.categoryId])
const expandedChapterIds = new Set<string>([currentLesson.chapterId])

const editor = createEditor(
  editorHost,
  source,
  (nextSource) => {
    source = nextSource
    latestAnalysis = undefined
    editor.dispatch(setDiagnostics(editor.state, []))
    liveAnalysis.schedule(source)
  },
  (position) => analysisHoverAt(latestAnalysis, source, position)
)

const liveAnalysis: LiveAnalysisController = createLiveAnalysis({
  analyze: analyzeSingleFile,
  onPending: () => {
    if (!runButton.disabled) setStatus("running", "Analyzing…")
  },
  onError: (error) => {
    if (runButton.disabled) return
    setStatus(
      "error",
      error instanceof Error ? error.message : "Analysis failed"
    )
  },
  apply: (analysis, analyzedSource) => {
    latestAnalysis = analysis
    const diagnostics = analysis.diagnostics.diagnostics
    editor.dispatch(
      setDiagnostics(editor.state, [
        ...toEditorDiagnostics(analyzedSource, diagnostics),
      ])
    )
    if (runButton.disabled) return
    if (diagnostics.length > 0) {
      showDiagnostics(diagnostics, analyzedSource)
      setStatus("error", `${diagnostics.length} diagnostic(s)`)
    } else {
      if (output.dataset.liveDiagnostics === "true") {
        showTextOutput("No diagnostics. Runでprogramを実行できます。")
      }
      setStatus("ready", "Analysis ready")
    }
  },
})

connectPreviewFullscreen(outputSection, fullscreenButton)
loadLesson(currentLesson.id, "replace")

menuButton.addEventListener("click", () => {
  const open = navigation.dataset.mobileOpen !== "true"
  setNavigationOpen(open)
})
menuCloseButton.addEventListener("click", () => {
  setNavigationOpen(false, { returnFocus: true })
})
navigation.addEventListener("keydown", handleNavigationKeydown)
collapsibleNavigationQuery.addEventListener("change", syncNavigationViewport)
mobileNavigationQuery.addEventListener("change", syncNavigationViewport)
previousButton.addEventListener("click", () => moveLesson(-1))
nextButton.addEventListener("click", () => moveLesson(1))
runButton.addEventListener("click", () => void run())
resetButton.addEventListener("click", () => resetLesson())
exerciseButton.addEventListener("click", () => {
  if (currentLesson.format === undefined) return
  loadSourceVariant(
    currentLesson.exerciseSource,
    "課題sourceを開きました。Runで期待結果と比べられます。",
    "Exercise ready"
  )
})
diagnosticButton.addEventListener("click", () => {
  if (currentLesson.format === undefined) return
  loadSourceVariant(
    currentLesson.diagnosticSource,
    "失敗例を開きました。compiler diagnosticを確認してください。",
    "Diagnostic example ready"
  )
})
formatButton.addEventListener("click", () => void formatSource())
showTextButton.addEventListener("click", () => closeInteractivePreview())
showPreviewButton.addEventListener("click", () => setOutputMode("html"))
document.addEventListener("keydown", (event) => {
  if (event.key !== "Enter" || (!event.metaKey && !event.ctrlKey)) return
  event.preventDefault()
  if (!runButton.disabled) void run()
})
window.addEventListener("popstate", () => {
  const lessonId = new URL(window.location.href).searchParams.get("lesson")
  loadLesson(findTourLesson(lessonId).id, "none")
})
window.addEventListener("beforeunload", () => cancelActiveExecution())

function renderNavigation(): void {
  const model = buildTourNavigationModel(
    tourCategories,
    tourChapters,
    tourLessons,
    currentLesson.id,
    progress.completedLessonIds
  )
  chapterHost.replaceChildren(
    ...model.categories.map((category) => {
      const section = document.createElement("section")
      section.className = "tour-category"
      section.dataset.categoryId = category.id
      const heading = document.createElement("h2")
      const categoryToggle = disclosureButton(
        category.title,
        category.progress,
        `tour-category-panel-${category.id}`,
        expandedCategoryIds,
        category.id,
        "tour-category-toggle"
      )
      heading.append(categoryToggle)
      const categoryPanel = document.createElement("div")
      categoryPanel.id = `tour-category-panel-${category.id}`
      categoryPanel.className = "tour-category-panel"
      categoryPanel.hidden = !expandedCategoryIds.has(category.id)
      categoryPanel.setAttribute("role", "group")
      categoryPanel.setAttribute("aria-label", `${category.title}のchapter`)
      const overview = document.createElement("div")
      overview.className = "tour-category-overview"
      const description = document.createElement("p")
      description.textContent = category.summary
      const goalLabel = document.createElement("strong")
      goalLabel.textContent = "到達目標"
      const categoryGoal = document.createElement("p")
      categoryGoal.textContent = category.goal
      const resume = document.createElement("button")
      resume.type = "button"
      resume.className = "tour-category-resume"
      resume.disabled = category.resumeLessonId === ""
      resume.textContent =
        category.progress.completed === category.progress.total
          ? `復習する · ${category.resumeLessonTitle}`
          : `続きから · ${category.resumeLessonTitle}`
      resume.setAttribute(
        "aria-label",
        `${category.title}を${category.progress.completed === category.progress.total ? "復習する" : "続きから再開する"}、${category.resumeLessonTitle}`
      )
      resume.addEventListener("click", () =>
        loadLesson(category.resumeLessonId, "push", true)
      )
      overview.append(description, goalLabel, categoryGoal, resume)
      const chapters = document.createElement("div")
      chapters.className = "tour-category-chapters"
      for (const chapter of category.chapters) {
        const chapterSection = document.createElement("section")
        chapterSection.className = "tour-chapter"
        chapterSection.dataset.chapterId = chapter.id
        const chapterHeading = document.createElement("h3")
        const chapterToggle = disclosureButton(
          chapter.title,
          chapter.progress,
          `tour-chapter-panel-${chapter.id}`,
          expandedChapterIds,
          chapter.id,
          "tour-chapter-toggle"
        )
        chapterHeading.append(chapterToggle)
        const chapterPanel = document.createElement("div")
        chapterPanel.id = `tour-chapter-panel-${chapter.id}`
        chapterPanel.className = "tour-chapter-panel"
        chapterPanel.hidden = !expandedChapterIds.has(chapter.id)
        chapterPanel.setAttribute("role", "group")
        chapterPanel.setAttribute("aria-label", `${chapter.title}のlesson`)
        const chapterSummary = document.createElement("p")
        chapterSummary.textContent = chapter.summary
        const list = document.createElement("ol")
        for (const lesson of chapter.lessons) {
          const item = document.createElement("li")
          const button = document.createElement("button")
          button.type = "button"
          button.className = "tour-lesson-link"
          button.dataset.lessonId = lesson.id
          button.dataset.state = lesson.state
          button.dataset.completed = String(
            progress.completedLessonIds.includes(lesson.id)
          )
          if (lesson.state === "current") {
            button.setAttribute("aria-current", "step")
          }
          button.addEventListener("click", () =>
            loadLesson(lesson.id, "push", true)
          )
          const number = document.createElement("span")
          number.className = "tour-lesson-number"
          number.textContent = String(lesson.position).padStart(2, "0")
          const title = document.createElement("span")
          title.className = "tour-lesson-link-title"
          title.textContent = lesson.title
          const state = document.createElement("span")
          state.className = "tour-lesson-state"
          state.setAttribute("aria-hidden", "true")
          state.textContent = lessonStateLabel(
            lesson.state,
            progress.completedLessonIds.includes(lesson.id)
          )
          button.setAttribute(
            "aria-label",
            `${number.textContent} ${lesson.title}、${state.textContent}`
          )
          button.append(number, title, state)
          item.append(button)
          list.append(item)
        }
        chapterPanel.append(chapterSummary, list)
        chapterSection.append(chapterHeading, chapterPanel)
        chapters.append(chapterSection)
      }
      categoryPanel.append(overview, chapters)
      section.append(heading, categoryPanel)
      return section
    })
  )
}

function disclosureButton(
  title: string,
  summary: TourProgressSummary,
  controlsId: string,
  expandedIds: Set<string>,
  id: string,
  className: string
): HTMLButtonElement {
  const button = document.createElement("button")
  button.type = "button"
  button.className = className
  button.setAttribute("aria-controls", controlsId)
  button.setAttribute("aria-expanded", String(expandedIds.has(id)))
  button.setAttribute(
    "aria-label",
    `${title}、進捗 ${progressText(summary)}、${expandedIds.has(id) ? "折りたたむ" : "展開する"}`
  )
  const icon = document.createElement("span")
  icon.className = "tour-disclosure-icon"
  icon.setAttribute("aria-hidden", "true")
  const label = document.createElement("span")
  label.className = "tour-disclosure-title"
  label.textContent = title
  const count = document.createElement("span")
  count.className = "tour-disclosure-count"
  count.textContent = progressText(summary)
  const progress = document.createElement("progress")
  progress.max = summary.total
  progress.value = summary.completed
  progress.textContent = progressText(summary)
  progress.setAttribute("aria-label", `${title}の進捗 ${progressText(summary)}`)
  button.append(icon, label, count, progress)
  button.addEventListener("click", () =>
    setDisclosure(button, controlsId, expandedIds, id, !expandedIds.has(id))
  )
  button.addEventListener("keydown", (event) => {
    if (event.key !== "ArrowLeft" && event.key !== "ArrowRight") return
    event.preventDefault()
    setDisclosure(
      button,
      controlsId,
      expandedIds,
      id,
      event.key === "ArrowRight"
    )
  })
  return button
}

function setDisclosure(
  button: HTMLButtonElement,
  controlsId: string,
  expandedIds: Set<string>,
  id: string,
  expanded: boolean
): void {
  if (expanded) expandedIds.add(id)
  else expandedIds.delete(id)
  button.setAttribute("aria-expanded", String(expanded))
  const panel = document.getElementById(controlsId)
  if (panel !== null) panel.hidden = !expanded
  const title =
    button.querySelector<HTMLElement>(".tour-disclosure-title")?.textContent ??
    ""
  const count =
    button.querySelector<HTMLElement>(".tour-disclosure-count")?.textContent ??
    ""
  button.setAttribute(
    "aria-label",
    `${title}、進捗 ${count}、${expanded ? "折りたたむ" : "展開する"}`
  )
}

function lessonStateLabel(
  state: "current" | "completed" | "unstarted",
  completed: boolean
): string {
  if (state === "current") return completed ? "現在 · 完了" : "現在"
  return state === "completed" ? "完了" : "未着手"
}

function progressText(summary: TourProgressSummary): string {
  return `${summary.completed} / ${summary.total}`
}

function renderNeighborButton(
  button: HTMLButtonElement,
  direction: "previous" | "next",
  neighbor: TourLessonNeighbor | undefined
): void {
  button.disabled = neighbor === undefined
  const directionLabel = direction === "previous" ? "← 前へ" : "次へ →"
  if (neighbor === undefined) {
    button.textContent =
      direction === "previous" ? "最初のlessonです" : "最後のlessonです"
    button.removeAttribute("aria-label")
    return
  }
  const directionText = document.createElement("span")
  directionText.className = "tour-neighbor-direction"
  directionText.textContent = directionLabel
  const title = document.createElement("span")
  title.className = "tour-neighbor-title"
  title.textContent = neighbor.title
  const boundary = document.createElement("span")
  boundary.className = "tour-neighbor-boundary"
  boundary.hidden = !neighbor.crossesCategory
  boundary.textContent = `${direction === "previous" ? "前" : "次"}のcategory · ${neighbor.categoryTitle}`
  button.replaceChildren(directionText, title, boundary)
  button.setAttribute(
    "aria-label",
    `${direction === "previous" ? "前" : "次"}のlesson、${neighbor.title}${
      neighbor.crossesCategory ? `、${boundary.textContent}` : ""
    }`
  )
}

function loadLesson(
  lessonId: string,
  historyMode: "push" | "replace" | "none",
  selectedFromNavigation = false
): void {
  const navigationWasOpen = navigation.dataset.mobileOpen === "true"
  cancelActiveExecution()
  currentLesson = findTourLesson(lessonId)
  expandedCategoryIds.add(currentLesson.categoryId)
  expandedChapterIds.add(currentLesson.chapterId)
  progress = visitTourLesson(progress, currentLesson.id)
  persistProgress()
  source = currentLesson.source
  outputMode = currentLesson.outputMode
  latestAnalysis = undefined
  stdinInput.value = currentLesson.stdin
  inputSection.hidden = currentLesson.stdin === ""
  replaceEditorSource(editor, source)
  editor.dispatch(setDiagnostics(editor.state, []))
  showTextOutput("Runを押すと結果がここに表示されます。")
  renderLesson()
  liveAnalysis.schedule(source)
  setStatus("ready", "Lesson ready")
  setNavigationOpen(false, {
    restoreScroll: !(selectedFromNavigation && navigationWasOpen),
  })
  if (selectedFromNavigation && navigationWasOpen) {
    requestAnimationFrame(() => {
      lesson.scrollIntoView({ block: "start" })
      lessonTitle.focus({ preventScroll: true })
    })
  }
  if (historyMode !== "none") updateLessonUrl(historyMode)
}

function renderLesson(): void {
  const category = tourCategories.find(
    ({ id }) => id === currentLesson.categoryId
  )
  const chapter = tourChapters.find(({ id }) => id === currentLesson.chapterId)
  const navigationModel = buildTourNavigationModel(
    tourCategories,
    tourChapters,
    tourLessons,
    currentLesson.id,
    progress.completedLessonIds
  )
  const neighbors = tourLessonNeighbors(
    tourCategories,
    tourLessons,
    currentLesson.id
  )
  chapterLabel.textContent = [category?.title, chapter?.title]
    .filter((label) => label !== undefined)
    .join(" / ")
  lessonTitle.textContent = currentLesson.title
  lessonSummary.textContent = currentLesson.summary
  goal.textContent = currentLesson.goal
  renderGuideMarkdown(lessonGuide, currentLesson.guide)
  challenge.textContent = currentLesson.challenge
  focusList.replaceChildren(
    ...currentLesson.focus.map((focus) => listItem(focus))
  )
  topicList.replaceChildren(
    ...currentLesson.introducedSurfaces.map((topic) => listItem(topic))
  )
  renderLessonFormat(currentLesson.format)
  stepLabel.textContent = `Step ${currentLesson.position} / ${tourLessons.length}`
  progressBar.max = navigationModel.progress.total
  progressBar.value = navigationModel.progress.completed
  progressBar.textContent = progressText(navigationModel.progress)
  progressBar.setAttribute(
    "aria-label",
    `Tour全体の進捗 ${progressText(navigationModel.progress)}`
  )
  progressLabel.textContent = `${navigationModel.progress.completed} completed`
  renderNeighborButton(previousButton, "previous", neighbors.previous)
  renderNeighborButton(nextButton, "next", neighbors.next)
  renderNavigation()
}

function renderLessonFormat(format: TourLessonFormat | undefined): void {
  const structured = format !== undefined
  guideSection.hidden = structured
  trySection.hidden = structured
  for (const section of [
    prerequisiteSection,
    runSection,
    walkthroughSection,
    introducedSection,
    exerciseSection,
    diagnosticSection,
    recapSection,
    nextSection,
  ]) {
    section.hidden = !structured
  }
  if (format === undefined) return

  renderTourInline(prerequisiteCopy, format.prerequisite)
  prerequisiteList.replaceChildren(
    ...currentLesson.prerequisites.map((id) => {
      const prerequisite = tourLessons.find((lesson) => lesson.id === id)
      return listItem(
        prerequisite === undefined
          ? id
          : `${String(prerequisite.position).padStart(2, "0")} ${prerequisite.title}`
      )
    })
  )
  prerequisiteList.hidden = currentLesson.prerequisites.length === 0

  runCopy.textContent = currentLesson.interactive
    ? "Runするとbrowser Previewが起動します。表示と操作を確認してください。"
    : "Runすると次の結果がOutputへ表示されます。"
  expectedOutput.hidden = currentLesson.interactive
  expectedOutput.textContent = currentLesson.expectedOutput

  walkthrough.replaceChildren(
    ...format.walkthrough.map((step) => walkthroughCard(step))
  )
  introduced.replaceChildren(
    ...format.introduced.flatMap((surface) => {
      const term = document.createElement("dt")
      const kind = document.createElement("span")
      kind.textContent = surface.kind
      term.append(kind, document.createTextNode(surface.name))
      const definition = document.createElement("dd")
      renderTourInline(definition, surface.body)
      return [term, definition]
    })
  )

  renderTourInline(exerciseCopy, format.exercise.instruction)
  exerciseOutput.textContent = currentLesson.exerciseExpectedOutput
  diagnosticHeading.textContent = format.diagnostic.heading
  renderTourInline(diagnosticCopy, format.diagnostic.body)
  diagnosticOutput.textContent = currentLesson.diagnosticOutput
  recapList.replaceChildren(...format.recap.map(inlineListItem))
  renderTourInline(nextCopy, format.next.body)
  const notes = format.notes ?? []
  notesList.hidden = notes.length === 0
  notesList.replaceChildren(...notes.map(inlineListItem))
}

function walkthroughCard(step: TourWalkthroughStep): HTMLElement {
  const card = document.createElement("article")
  card.className = "tour-walkthrough-card"
  const heading = document.createElement("div")
  heading.className = "tour-walkthrough-heading"
  const title = document.createElement("h3")
  title.textContent = step.heading
  const rangeButton = document.createElement("button")
  rangeButton.type = "button"
  const label = sourceRangeLabel(step.sourceRange)
  rangeButton.textContent = label
  rangeButton.setAttribute("aria-label", `${label}をlesson editorで選択する`)
  rangeButton.addEventListener("click", () =>
    selectCanonicalSourceRange(step.sourceRange)
  )
  heading.append(title, rangeButton)
  const body = document.createElement("p")
  renderTourInline(body, step.body)
  const excerpt = document.createElement("pre")
  const code = document.createElement("code")
  code.textContent = sourceExcerpt(currentLesson.source, step.sourceRange)
  excerpt.append(code)
  card.append(heading, body, excerpt)
  return card
}

function sourceRangeLabel(range: TourSourceRange): string {
  return range.startLine === range.endLine
    ? `L${range.startLine}`
    : `L${range.startLine}–${range.endLine}`
}

function sourceExcerpt(sourceText: string, range: TourSourceRange): string {
  return sourceText
    .split(/\r?\n/u)
    .slice(range.startLine - 1, range.endLine)
    .join("\n")
}

function selectCanonicalSourceRange(range: TourSourceRange): void {
  if (source !== currentLesson.source) {
    source = currentLesson.source
    replaceEditorSource(editor, source)
    liveAnalysis.schedule(source)
  }
  const start = editor.state.doc.line(range.startLine)
  const end = editor.state.doc.line(range.endLine)
  editor.dispatch({
    selection: { anchor: start.from, head: end.to },
    scrollIntoView: true,
  })
  editor.focus()
  if (mobileNavigationQuery.matches) {
    lab.scrollIntoView({ block: "start" })
  }
}

type NavigationCloseOptions = Readonly<{
  restoreScroll?: boolean
  returnFocus?: boolean
}>

function setNavigationOpen(
  requestedOpen: boolean,
  options: NavigationCloseOptions = {}
): void {
  const collapsible = collapsibleNavigationQuery.matches
  const wasOpen = navigation.dataset.mobileOpen === "true"
  const open = collapsible && requestedOpen
  navigation.dataset.mobileOpen = String(open)
  menuButton.setAttribute("aria-expanded", String(open))

  if (!collapsible) {
    navigation.inert = false
    setNavigationBackgroundInert(false)
    navigation.removeAttribute("aria-hidden")
    navigation.removeAttribute("aria-modal")
    navigation.removeAttribute("role")
    document.body.classList.remove("tour-navigation-sheet-open")
    return
  }

  navigation.inert = !open
  setNavigationBackgroundInert(open)
  navigation.setAttribute("aria-hidden", String(!open))
  if (open) {
    navigation.setAttribute("role", "dialog")
    navigation.setAttribute("aria-modal", "true")
    if (mobileNavigationQuery.matches) {
      navigationBackgroundScrollTop = document.body.scrollTop
      document.body.classList.add("tour-navigation-sheet-open")
    }
    requestAnimationFrame(() => menuCloseButton.focus({ preventScroll: true }))
    return
  }

  navigation.removeAttribute("aria-modal")
  navigation.removeAttribute("role")
  document.body.classList.remove("tour-navigation-sheet-open")
  if (
    wasOpen &&
    mobileNavigationQuery.matches &&
    options.restoreScroll !== false
  ) {
    document.body.scrollTop = navigationBackgroundScrollTop
  }
  if (wasOpen && options.returnFocus === true) {
    menuButton.focus({ preventScroll: true })
  }
}

function syncNavigationViewport(): void {
  const open = navigation.dataset.mobileOpen === "true"
  if (!collapsibleNavigationQuery.matches) {
    setNavigationOpen(false)
    return
  }

  navigation.inert = !open
  setNavigationBackgroundInert(open)
  navigation.setAttribute("aria-hidden", String(!open))
  if (open) {
    navigation.setAttribute("role", "dialog")
    navigation.setAttribute("aria-modal", "true")
  }
  const shouldLockBackground = open && mobileNavigationQuery.matches
  if (
    shouldLockBackground &&
    !document.body.classList.contains("tour-navigation-sheet-open")
  ) {
    navigationBackgroundScrollTop = document.body.scrollTop
  }
  document.body.classList.toggle(
    "tour-navigation-sheet-open",
    shouldLockBackground
  )
}

function setNavigationBackgroundInert(inert: boolean): void {
  for (const element of [topbar, menuButton, lesson, lab]) {
    element.inert = inert
  }
}

function handleNavigationKeydown(event: KeyboardEvent): void {
  if (navigation.dataset.mobileOpen !== "true") return
  if (event.key === "Escape") {
    event.preventDefault()
    setNavigationOpen(false, { returnFocus: true })
    return
  }
  if (event.key !== "Tab") return

  const focusable = [
    menuCloseButton,
    ...chapterHost.querySelectorAll<HTMLButtonElement>("button:not(:disabled)"),
  ]
  const first = focusable[0]
  const last = focusable.at(-1)
  if (first === undefined || last === undefined) return
  if (event.shiftKey && document.activeElement === first) {
    event.preventDefault()
    last.focus()
  } else if (!event.shiftKey && document.activeElement === last) {
    event.preventDefault()
    first.focus()
  }
}

function listItem(text: string): HTMLLIElement {
  const item = document.createElement("li")
  item.textContent = text
  return item
}

function inlineListItem(text: TourInlineRichText): HTMLLIElement {
  const item = document.createElement("li")
  renderTourInline(item, text)
  return item
}

function renderTourInline(
  target: HTMLElement,
  source: TourInlineRichText
): void {
  target.classList.add("tour-inline-rich-text")
  renderGuideInline(target, source)
}

function moveLesson(offset: number): void {
  const index = tourLessons.findIndex(({ id }) => id === currentLesson.id)
  const next = tourLessons[index + offset]
  if (next !== undefined) loadLesson(next.id, "push")
}

function updateLessonUrl(mode: "push" | "replace"): void {
  const url = new URL(window.location.href)
  url.searchParams.set("lesson", currentLesson.id)
  history[mode === "push" ? "pushState" : "replaceState"]({}, "", url)
}

function persistProgress(): void {
  try {
    saveTourProgress(localStorage, progress)
  } catch {
    // The Tour remains usable when private browsing denies local storage.
  }
}

function completeCurrentLesson(): void {
  progress = completeTourLesson(progress, currentLesson.id)
  persistProgress()
  renderLesson()
}

function resetLesson(): void {
  cancelActiveExecution()
  source = currentLesson.source
  outputMode = currentLesson.outputMode
  stdinInput.value = currentLesson.stdin
  replaceEditorSource(editor, source)
  editor.dispatch(setDiagnostics(editor.state, []))
  showTextOutput("Lessonを初期状態へ戻しました。")
  liveAnalysis.schedule(source)
  setStatus("ready", "Lesson reset")
  editor.focus()
}

function loadSourceVariant(
  variantSource: string,
  outputMessage: string,
  statusMessage: string
): void {
  cancelActiveExecution()
  source = variantSource
  outputMode = "text"
  replaceEditorSource(editor, source)
  editor.dispatch(setDiagnostics(editor.state, []))
  showTextOutput(outputMessage)
  liveAnalysis.schedule(source)
  setStatus("ready", statusMessage)
  editor.focus()
  if (mobileNavigationQuery.matches) {
    lab.scrollIntoView({ block: "start" })
  }
}

async function formatSource(): Promise<void> {
  const requestedSource = source
  setStatus("running", "Formatting…")
  try {
    const formatted = await formatSingleFile(requestedSource)
    if (source !== requestedSource) return
    if (formatted.status === "failure") {
      const diagnostics = formatted.diagnostics.diagnostics
      editor.dispatch(
        setDiagnostics(editor.state, [
          ...toEditorDiagnostics(requestedSource, diagnostics),
        ])
      )
      showDiagnostics(diagnostics, requestedSource)
      setStatus("error", `Cannot format: ${diagnostics.length} diagnostic(s)`)
      return
    }
    if (formatted.changed) {
      source = formatted.source
      replaceEditorSource(editor, source)
      setStatus("success", "Formatted")
    } else {
      setStatus("success", "Already formatted")
    }
  } catch (error) {
    if (source !== requestedSource) return
    setStatus(
      "error",
      error instanceof Error ? error.message : "Formatting failed"
    )
  } finally {
    editor.focus()
  }
}

async function run(): Promise<void> {
  cancelActiveExecution()
  const revision = runRevision
  runButton.disabled = true
  showTextOutput("Compiling with the shared Rust driver…")
  setStatus("running", "Compiling…")
  try {
    const compiled = await compileSingleFile(source)
    if (revision !== runRevision) return
    const diagnostics = compiled.diagnostics.diagnostics
    editor.dispatch(
      setDiagnostics(editor.state, [
        ...toEditorDiagnostics(source, diagnostics),
      ])
    )
    if (compiled.status === "failure") {
      showDiagnostics(diagnostics)
      setStatus("error", `${diagnostics.length} diagnostic(s)`)
      return
    }
    if (!compiled.entry) {
      showTextOutput(
        compiled.entryError ?? "Compile succeeded. No executable main found."
      )
      setStatus("ready", "Compile succeeded")
      return
    }
    setStatus("running", "Running…")
    const needsDom = compiled.entry.environment.some(
      (binding) => binding.service === "dom"
    )
    const domDocument = needsDom ? await prepareInteractivePreview() : undefined
    if (revision !== runRevision) return
    const execution = await startGeneratedModule(
      compiled.generated.typescript,
      compiled.entry,
      stdinInput.value,
      {
        ...(domDocument === undefined ? {} : { domDocument }),
        onDomMounted: () => {
          if (revision !== runRevision) return
          output.textContent = "Interactive preview is running."
          setOutputMode("html")
          setStatus("success", "Interactive")
          completeCurrentLesson()
        },
      }
    )
    if (revision !== runRevision) {
      await execution.cancel()
      return
    }
    activeExecution = execution
    void execution.completion.then(
      (completion) => {
        if (revision !== runRevision || activeExecution !== execution) return
        activeExecution = undefined
        if (completion.kind === "cancelled") return
        showExecutionOutput(completion.result.stdout)
        setStatus("success", "Completed")
        completeCurrentLesson()
      },
      (error: unknown) => {
        if (revision !== runRevision || activeExecution !== execution) return
        activeExecution = undefined
        showTextOutput(error instanceof Error ? error.message : String(error))
        setStatus("error", "Execution failed")
      }
    )
  } catch (error) {
    if (revision !== runRevision) return
    showTextOutput(error instanceof Error ? error.message : String(error))
    setStatus("error", "Execution failed")
  } finally {
    runButton.disabled = false
  }
}

function cancelActiveExecution(): void {
  runRevision += 1
  const execution = activeExecution
  activeExecution = undefined
  if (execution !== undefined) void execution.cancel()
}

function showExecutionOutput(stdout: string): void {
  output.className = "output-text"
  delete output.dataset.liveDiagnostics
  output.textContent = stdout || "Program completed with no output."
  renderHtmlPreview(stdout)
  setOutputMode(outputMode)
}

function showTextOutput(message: string): void {
  output.className = "output-text"
  delete output.dataset.liveDiagnostics
  output.textContent = message
  clearHtmlPreview()
  renderOutputMode("text")
}

function showDiagnostics(
  diagnostics: readonly Diagnostic[],
  analyzedSource = source
): void {
  clearHtmlPreview()
  renderOutputMode("text")
  output.dataset.liveDiagnostics = "true"
  renderDiagnosticCards(output, diagnostics, analyzedSource, (byteRange) => {
    const range = utf8RangeToUtf16(analyzedSource, byteRange)
    editor.dispatch({
      selection: { anchor: range.from, head: range.to },
      scrollIntoView: true,
    })
    editor.focus()
  })
}

function renderHtmlPreview(html: string): void {
  clearHtmlPreview()
  if (html === "") return
  const url = URL.createObjectURL(
    new Blob([createPreviewDocument(html)], { type: "text/html" })
  )
  htmlPreviewUrl = url
  htmlPreview.addEventListener(
    "load",
    () => {
      if (htmlPreviewUrl !== url) return
      URL.revokeObjectURL(url)
      htmlPreviewUrl = undefined
    },
    { once: true }
  )
  htmlPreview.src = url
}

async function prepareInteractivePreview(): Promise<Document> {
  clearHtmlPreview()
  const url = URL.createObjectURL(
    new Blob([createPreviewDocument('<div id="app"></div>')], {
      type: "text/html",
    })
  )
  htmlPreviewUrl = url
  const loaded = new Promise<void>((resolve, reject) => {
    htmlPreview.addEventListener("load", () => resolve(), { once: true })
    htmlPreview.addEventListener(
      "error",
      () => reject(new Error("interactive preview failed to load")),
      { once: true }
    )
  })
  htmlPreview.src = url
  await loaded
  if (htmlPreviewUrl === url) {
    URL.revokeObjectURL(url)
    htmlPreviewUrl = undefined
  }
  const previewDocument = htmlPreview.contentDocument
  if (previewDocument === null) {
    throw new Error("interactive preview document is unavailable")
  }
  setOutputMode("html")
  return previewDocument
}

function clearHtmlPreview(): void {
  if (htmlPreviewUrl !== undefined) URL.revokeObjectURL(htmlPreviewUrl)
  htmlPreviewUrl = undefined
  htmlPreview.removeAttribute("src")
}

function setOutputMode(mode: "text" | "html"): void {
  outputMode = mode
  renderOutputMode(mode)
}

function closeInteractivePreview(): void {
  const wasRunning = activeExecution !== undefined
  const hasPreview =
    !htmlPreview.hidden || htmlPreview.getAttribute("src") !== null
  if (hasPreview) {
    cancelActiveExecution()
    clearHtmlPreview()
  }
  setOutputMode("text")
  if (wasRunning) setStatus("ready", "Preview closed")
}

function renderOutputMode(mode: "text" | "html"): void {
  const showHtml = mode === "html"
  output.hidden = showHtml
  htmlPreview.hidden = !showHtml
  showTextButton.setAttribute("aria-pressed", String(!showHtml))
  showPreviewButton.setAttribute("aria-pressed", String(showHtml))
}

function setStatus(
  state: "ready" | "running" | "success" | "error",
  message: string
): void {
  statusDot.dataset.state = state
  statusText.textContent = message
}
