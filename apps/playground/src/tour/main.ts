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
import { connectPreviewFullscreen } from "../ui/preview-fullscreen"
import { findTourLesson, tourChapters, tourLessons } from "./curriculum"
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
const navigation = requiredElement("#tour-navigation", HTMLElement)
const stepLabel = requiredElement("#tour-step-label", HTMLElement)
const progressBar = requiredElement("#tour-progress", HTMLProgressElement)
const progressLabel = requiredElement("#tour-progress-label", HTMLElement)
const chapterLabel = requiredElement("#tour-chapter-label", HTMLElement)
const lessonTitle = requiredElement("#tour-lesson-title", HTMLElement)
const lessonSummary = requiredElement("#tour-lesson-summary", HTMLElement)
const focusList = requiredElement("#tour-focus-list", HTMLUListElement)
const lessonGuide = requiredElement("#tour-guide", HTMLElement)
const challenge = requiredElement("#tour-challenge", HTMLElement)
const topicList = requiredElement("#tour-topic-list", HTMLUListElement)
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
renderNavigation()
loadLesson(currentLesson.id, "replace")

menuButton.addEventListener("click", () => {
  const open = navigation.dataset.mobileOpen !== "true"
  navigation.dataset.mobileOpen = String(open)
  menuButton.setAttribute("aria-expanded", String(open))
})
previousButton.addEventListener("click", () => moveLesson(-1))
nextButton.addEventListener("click", () => moveLesson(1))
runButton.addEventListener("click", () => void run())
resetButton.addEventListener("click", () => resetLesson())
formatButton.addEventListener("click", () => void formatSource())
showTextButton.addEventListener("click", () => setOutputMode("text"))
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
  chapterHost.replaceChildren(
    ...tourChapters.map((chapter) => {
      const section = document.createElement("section")
      section.className = "tour-chapter"
      const heading = document.createElement("h2")
      heading.textContent = chapter.title
      const description = document.createElement("p")
      description.textContent = chapter.summary
      const list = document.createElement("ol")
      for (const lesson of tourLessons.filter(
        ({ chapter: chapterId }) => chapterId === chapter.id
      )) {
        const item = document.createElement("li")
        const button = document.createElement("button")
        button.type = "button"
        button.dataset.lessonId = lesson.id
        button.addEventListener("click", () => loadLesson(lesson.id, "push"))
        const number = document.createElement("span")
        number.className = "tour-lesson-number"
        number.textContent = String(lesson.order).padStart(2, "0")
        const title = document.createElement("span")
        title.className = "tour-lesson-link-title"
        title.textContent = lesson.title
        const state = document.createElement("span")
        state.className = "tour-lesson-state"
        state.setAttribute("aria-hidden", "true")
        button.append(number, title, state)
        item.append(button)
        list.append(item)
      }
      section.append(heading, description, list)
      return section
    })
  )
}

function loadLesson(
  lessonId: string,
  historyMode: "push" | "replace" | "none"
): void {
  cancelActiveExecution()
  currentLesson = findTourLesson(lessonId)
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
  navigation.dataset.mobileOpen = "false"
  menuButton.setAttribute("aria-expanded", "false")
  if (historyMode !== "none") updateLessonUrl(historyMode)
}

function renderLesson(): void {
  const chapter = tourChapters.find(({ id }) => id === currentLesson.chapter)
  const index = tourLessons.findIndex(({ id }) => id === currentLesson.id)
  chapterLabel.textContent = chapter?.title ?? ""
  lessonTitle.textContent = currentLesson.title
  lessonSummary.textContent = currentLesson.summary
  lessonGuide.textContent = currentLesson.guide.trim()
  challenge.textContent = currentLesson.challenge
  focusList.replaceChildren(
    ...currentLesson.focus.map((focus) => listItem(focus))
  )
  topicList.replaceChildren(
    ...currentLesson.introduces.map((topic) => listItem(topic))
  )
  stepLabel.textContent = `Step ${currentLesson.order} / ${tourLessons.length}`
  const completed = progress.completedLessonIds.length
  progressBar.max = tourLessons.length
  progressBar.value = completed
  progressBar.textContent = `${completed} / ${tourLessons.length}`
  progressLabel.textContent = `${completed} completed`
  previousButton.disabled = index <= 0
  nextButton.disabled = index >= tourLessons.length - 1
  for (const button of chapterHost.querySelectorAll<HTMLButtonElement>(
    "[data-lesson-id]"
  )) {
    const active = button.dataset.lessonId === currentLesson.id
    const complete = progress.completedLessonIds.includes(
      button.dataset.lessonId ?? ""
    )
    if (active) button.setAttribute("aria-current", "step")
    else button.removeAttribute("aria-current")
    button.dataset.completed = String(complete)
    const state = button.querySelector<HTMLElement>(".tour-lesson-state")
    if (state !== null) state.textContent = complete ? "✓" : ""
  }
}

function listItem(text: string): HTMLLIElement {
  const item = document.createElement("li")
  item.textContent = text
  return item
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
  stdinInput.value = currentLesson.stdin
  replaceEditorSource(editor, source)
  editor.dispatch(setDiagnostics(editor.state, []))
  showTextOutput("Lessonを初期状態へ戻しました。")
  liveAnalysis.schedule(source)
  setStatus("ready", "Lesson reset")
  editor.focus()
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
      (result) => {
        if (revision !== runRevision || activeExecution !== execution) return
        activeExecution = undefined
        showExecutionOutput(result.stdout)
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
