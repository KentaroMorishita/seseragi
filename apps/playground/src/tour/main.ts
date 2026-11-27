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
import {
  createEditor,
  replaceEditorSource,
  setEditorLinePins,
  setEditorWhitespaceVisible,
} from "../editor/create-editor"
import { resolveEditorLineWidth } from "../editor/format-width"
import { highlightSeseragi } from "../editor/seseragi-language"
import { createEditorPreferencesStore } from "../preferences/editor-preferences"
import { createPreviewDocument } from "../preview-document"
import type { BrowserExecution } from "../runtime/browser-execution"
import {
  type IsolatedExecutionRequest,
  startIsolatedProject,
} from "../runtime/isolated-execution"
import {
  contractNeedsNetwork,
  createNetworkPermissionStore,
  htmlNeedsNetwork,
} from "../runtime/network-permissions"
import { connectPreviewLinks } from "../runtime/preview-links"
import {
  isPreviewNavigationCancellation,
  previewNavigationCancelled,
} from "../runtime/preview-navigation"
import {
  allowFromNotice,
  connectNetworkNotice,
  createNetworkConsent,
  createNetworkGate,
  createNetworkSettings,
  type NetworkScope,
} from "../ui/network-consent"
import "../ui/design/index.css"
import "../ui/playground.css"
import {
  deepDiveArticles,
  deepDiveArticlesForTourLesson,
} from "../deep-dive/catalog"
import { deepDiveRelativeUrl } from "../deep-dive/route"
import { connectEditorSettings } from "../ui/editor-settings"
import { requiredElement } from "../ui/elements"
import { renderGuideInline, renderGuideMarkdown } from "../ui/guide-markdown"
import { connectOverflowMenu } from "../ui/overflow-menu"
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
import { connectTourPaneLayout } from "./pane-layout"
import {
  completeTourLesson,
  loadTourProgress,
  saveTourProgress,
  type TourProgress,
  visitTourLesson,
} from "./progress"
import "./tour.css"
import curriculumJson from "../../../../examples/tour/curriculum.json"
import type { PlaygroundSample } from "../samples"
import {
  connectLocaleController,
  JAPANESE_ONLY_LABEL,
  playgroundHref as localizedPlaygroundHref,
  type UiLocale,
  UNTRANSLATED_LESSON_NOTICE,
  uiMessages,
} from "../ui/i18n"
import { createReferenceCard } from "../ui/reference-card"
import { exampleEntries, lessonEntries } from "../ui/search-index"
import { connectSearchPalette } from "../ui/search-palette"
import { createThemeController } from "../ui/theme"
import { browserStorage, connectThemeButton } from "../ui/theme-button"
import {
  chapterExamplesForLesson,
  isChapterEnd,
  tourOpenInPlaygroundUrl,
} from "./chapter-examples"

const chapterHost = requiredElement("#tour-chapters", HTMLElement)
const menuButton = requiredElement("#tour-menu-button", HTMLButtonElement)
const menuCloseButton = requiredElement(
  "#tour-menu-close-button",
  HTMLButtonElement
)
const lessonFocusButton = requiredElement(
  "#tour-lesson-focus-button",
  HTMLButtonElement
)
const codeFocusButton = requiredElement(
  "#tour-code-focus-button",
  HTMLButtonElement
)
const navigation = requiredElement("#tour-navigation", HTMLElement)
const workspace = requiredElement(".tour-workspace", HTMLElement)
const lesson = requiredElement(".tour-lesson", HTMLElement)
const lessonResizer = requiredElement("#tour-lesson-resizer", HTMLElement)
const lab = requiredElement(".tour-lab", HTMLElement)
const outputResizer = requiredElement("#tour-output-resizer", HTMLElement)
const outputPaneToggle = requiredElement(
  "#tour-output-pane-toggle",
  HTMLButtonElement
)
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
const chapterProgress = requiredElement("#tour-chapter-progress", HTMLElement)
const trySteps = requiredElement("#tour-try-steps", HTMLOListElement)
const tryResetButton = requiredElement(
  "#tour-try-reset-button",
  HTMLButtonElement
)
const runCheck = requiredElement("#tour-run-check", HTMLElement)
const searchButton = requiredElement("#tour-search-button", HTMLButtonElement)
const themeButton = requiredElement("#tour-theme-button", HTMLButtonElement)
const themeSystemButton = requiredElement(
  "#tour-theme-system-button",
  HTMLButtonElement
)
const furtherDetails = requiredElement("#tour-further", HTMLDetailsElement)
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
const topicList = requiredElement("#tour-topic-list", HTMLUListElement)
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
const deepDiveSection = requiredElement("#tour-deep-dive-section", HTMLElement)
const deepDiveList = requiredElement("#tour-deep-dive-list", HTMLUListElement)
const previousButton = requiredElement(
  "#tour-previous-button",
  HTMLButtonElement
)
const nextButton = requiredElement("#tour-next-button", HTMLButtonElement)
const runButton = requiredElement("#tour-run-button", HTMLButtonElement)
const resetButton = requiredElement("#tour-reset-button", HTMLButtonElement)
const formatButton = requiredElement("#tour-format-button", HTMLButtonElement)
// エディターの設定のダイアログは、Search の Settings から開く。
const settingsButton = document.createElement("button")
const toolsButton = requiredElement("#tour-tools-button", HTMLButtonElement)
const toolsMenu = requiredElement("#tour-tools-menu", HTMLElement)
const editorHost = requiredElement("#tour-editor", HTMLDivElement)
const statusDot = requiredElement("#tour-status-dot", HTMLElement)
const statusText = requiredElement("#tour-status-text", HTMLElement)
const inputSection = requiredElement("#tour-input-section", HTMLElement)
const stdinInput = requiredElement("#tour-stdin-input", HTMLTextAreaElement)
const outputSection = requiredElement("#tour-output-section", HTMLElement)
const output = requiredElement("#tour-output", HTMLElement)
const htmlPreview = requiredElement("#tour-html-preview", HTMLIFrameElement)
// Preview の中で押された外部リンクを、確かめてから新しいタブで開く（#1375）
const previewLinks = connectPreviewLinks({
  frame: htmlPreview,
  notice: requiredElement("#tour-preview-link-notice", HTMLElement),
  createDocument: createPreviewDocument,
})
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
const localeController = connectLocaleController({
  window,
  jaButton: requiredElement("#lang-ja-button", HTMLButtonElement),
  enButton: requiredElement("#lang-en-button", HTMLButtonElement),
  banner: document.querySelector<HTMLElement>("#english-available-banner"),
  bannerSwitchButton: document.querySelector<HTMLButtonElement>(
    "#english-banner-switch"
  ),
  bannerDismissButton: document.querySelector<HTMLButtonElement>(
    "#english-banner-dismiss"
  ),
  playgroundLinks: Array.from(
    document.querySelectorAll<HTMLAnchorElement>(
      '.pg-brand, .pg-views a[href="../"]'
    )
  ),
  tourLinks: Array.from(
    document.querySelectorAll<HTMLAnchorElement>('.pg-views a[href="./"]')
  ),
})
// 外部通信の許可（#1371）。レッスンごとに覚える
const networkPermissions = createNetworkPermissionStore(browserStorage())
const networkNotice = connectNetworkNotice(
  requiredElement("#tour-preview-network-notice", HTMLElement),
  () =>
    allowFromNotice(
      networkPermissions,
      currentNetworkScope,
      networkNotice,
      () => void run()
    ),
  localeController.current
)
const networkGate = createNetworkGate({
  store: networkPermissions,
  consent: createNetworkConsent(document, localeController.current),
  notice: networkNotice,
  current: currentNetworkScope,
  rerun: () => void run(),
  onRevoke: () => {
    cancelActiveExecution()
    clearHtmlPreview()
  },
})
const networkSettings = createNetworkSettings(
  document,
  networkPermissions,
  localeController.current
)
let runNetworkAllowed = false
let staticPreviewRequest = 0
let source = currentLesson.source
let outputMode: "text" | "html" = currentLesson.outputMode
let latestAnalysis: AnalysisDocument | undefined
let activeExecution: BrowserExecution | undefined
let runRevision = 0
let htmlPreviewUrl: string | undefined
let activePreviewPreparation: AbortController | undefined
let navigationBackgroundScrollTop = 0
// Try it の手順の進み具合。レッスンを開き直すと最初から数える。
let tryState = { ran: false, changed: false }
const expandedCategoryIds = new Set<string>([currentLesson.categoryId])
const expandedChapterIds = new Set<string>([currentLesson.chapterId])

const editor = createEditor(
  editorHost,
  source,
  (nextSource) => {
    cancelActiveExecution()
    source = nextSource
    latestAnalysis = undefined
    editor.dispatch(setDiagnostics(editor.state, []))
    liveAnalysis.schedule(source)
  },
  (position) => analysisHoverAt(latestAnalysis, source, position)
)
const preferences = createEditorPreferencesStore(localStorage, window)
preferences.subscribe(({ showWhitespace }) =>
  setEditorWhitespaceVisible(editor, showWhitespace)
)
connectEditorSettings({
  button: settingsButton,
  returnFocus: searchButton,
  idPrefix: "tour",
  store: preferences,
})
connectOverflowMenu({ button: toolsButton, menu: toolsMenu })
const themeController = createThemeController({
  storage: browserStorage(),
  root: document.documentElement,
  query: window.matchMedia("(prefers-color-scheme: dark)"),
  eventTarget: window,
})
connectThemeButton(themeButton, themeController, themeSystemButton)

// Search（⌘K）は Playground と同じ。例は Playground で開き、レッスンはこの場で開く。
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
      `${localizedPlaygroundHref(localeController.current())}?example=${encodeURIComponent(id)}`
    )
    return undefined
  },
  onLesson: (id) => loadLesson(findTourLesson(id).id, "push", true),
  commands: [
    {
      id: "reset",
      title: "Reset lesson",
      detail: "このレッスンのコードを、最初の状態に戻す",
      keywords: "元に戻す",
      run: () => {
        resetLesson()
        return undefined
      },
    },
    ...(["light", "dark", "system"] as const).map((preference) => ({
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
    {
      id: "network-access",
      title: "Network access",
      detail: "外部の画像と通信を許可したレッスンや例を見て、取り消す",
      keywords: "network 外部通信 画像 許可 permission",
      run: () => {
        networkSettings.open(searchButton)
        return undefined
      },
    },
    {
      id: "editor-settings",
      title: "Editor settings",
      detail: "インデントの表示と、Format の行の幅を変える",
      keywords: "settings 設定 whitespace width",
      run: () => {
        settingsButton.click()
        return undefined
      },
    },
  ],
  renderApiDetail: (item) => createReferenceCard(document, item),
  locale: localeController.current,
})
searchButton.addEventListener("click", () =>
  search.open({ opener: searchButton })
)

const openPlaygroundButton = document.querySelector<HTMLButtonElement>(
  "#tour-open-playground-button"
)
const chapterExamplesSection = document.querySelector<HTMLElement>(
  "#tour-chapter-examples"
)
const chapterExamplesTitle = document.querySelector<HTMLElement>(
  "#tour-chapter-examples-title"
)
const chapterExamplesList = document.querySelector<HTMLElement>(
  "#tour-chapter-examples-list"
)
let cachedChapterSamples: readonly PlaygroundSample[] | undefined

const tourUntranslatedNotice = document.querySelector<HTMLElement>(
  "#tour-untranslated-notice"
)
const tourExpectedSummary = document.querySelector<HTMLElement>(
  "#tour-expected > summary"
)
const tourGuideTitle = document.querySelector<HTMLElement>("#tour-guide-title")
const tourFocusTitle = document.querySelector<HTMLElement>("#tour-focus-title")
const tourIntroducedTitle = document.querySelector<HTMLElement>(
  "#tour-introduced-title"
)
const tourDiagnosticTitle = document.querySelector<HTMLElement>(
  "#tour-diagnostic-title"
)
const tourRecapTitle = document.querySelector<HTMLElement>("#tour-recap-title")
const tourPrerequisiteTitle = document.querySelector<HTMLElement>(
  "#tour-prerequisite-title"
)
const tourNextTitle = document.querySelector<HTMLElement>("#tour-next-title")
const tourDeepDiveTitle = document.querySelector<HTMLElement>(
  "#tour-deep-dive-title"
)

const applyTourLocale = (nextLocale: UiLocale): void => {
  const messages = uiMessages(nextLocale)
  const otherMessages = uiMessages(nextLocale === "en" ? "ja" : "en")
  if (tourUntranslatedNotice !== null) {
    tourUntranslatedNotice.hidden = nextLocale !== "en"
    tourUntranslatedNotice.textContent =
      nextLocale === "en" ? UNTRANSLATED_LESSON_NOTICE : ""
  }
  if (tourExpectedSummary) {
    tourExpectedSummary.textContent = messages.tourExpectedSummary
  }
  if (tourGuideTitle) tourGuideTitle.textContent = messages.tourGuideTitle
  if (tourFocusTitle) tourFocusTitle.textContent = messages.tourFocusTitle
  if (tourIntroducedTitle) {
    tourIntroducedTitle.textContent = messages.tourIntroducedTitle
  }
  if (tourDiagnosticTitle) {
    tourDiagnosticTitle.textContent = messages.tourDiagnosticTitle
  }
  diagnosticButton.textContent = messages.tourDiagnosticButton
  if (tourRecapTitle) tourRecapTitle.textContent = messages.tourRecapTitle
  if (tourPrerequisiteTitle) {
    tourPrerequisiteTitle.textContent = messages.tourPrerequisiteTitle
  }
  if (tourNextTitle) tourNextTitle.textContent = messages.tourNextTitle
  if (tourDeepDiveTitle) {
    tourDeepDiveTitle.textContent = messages.tourDeepDiveTitle
  }
  if (chapterExamplesTitle) {
    chapterExamplesTitle.textContent = messages.tourChapterExamplesTitle
  }
  if (openPlaygroundButton) {
    openPlaygroundButton.title = messages.tourOpenInPlaygroundTitle
  }
  if (
    runCheck.textContent === otherMessages.tourRunCheckOk ||
    runCheck.textContent === messages.tourRunCheckOk
  ) {
    runCheck.textContent = messages.tourRunCheckOk
  }
  if (output.dataset.uiPlaceholder === "idle") {
    output.textContent = messages.outputPlaceholder
  } else if (output.dataset.uiPlaceholder === "no-errors") {
    output.textContent = messages.noErrorsRunPrompt
  }
  const untranslatedContent = [
    lessonTitle,
    lessonSummary,
    goal,
    lessonGuide,
    walkthrough,
    trySteps,
    focusList,
    introduced,
    diagnosticHeading,
    diagnosticCopy,
    recapList,
    prerequisiteCopy,
    prerequisiteList,
    nextCopy,
    notesList,
    deepDiveList,
  ]
  for (const element of untranslatedContent) {
    if (nextLocale === "en") element.setAttribute("lang", "ja")
    else element.removeAttribute("lang")
  }
  networkNotice.refreshLocale?.()
  search.refresh()
  renderChapterExamples()
}
applyTourLocale(localeController.current())
localeController.subscribe(applyTourLocale)

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
    search.setApiCatalog(analysis.standardLibrary)
    const diagnostics = analysis.diagnostics.diagnostics
    editor.dispatch(
      setDiagnostics(editor.state, [
        ...toEditorDiagnostics(analyzedSource, diagnostics),
      ])
    )
    if (runButton.disabled) return
    if (diagnostics.length > 0) {
      showDiagnostics(diagnostics, analyzedSource)
      setStatus("error", problemCount(diagnostics.length))
    } else {
      if (output.dataset.liveDiagnostics === "true") {
        showTextOutput(
          localeController.messages().noErrorsRunPrompt,
          "no-errors"
        )
      }
      setStatus("ready", "Analysis ready")
    }
  },
})

connectTourPaneLayout(
  {
    workspace,
    lessonResizer,
    lab,
    outputResizer,
    outputToggle: outputPaneToggle,
  },
  () => editor.requestMeasure()
)
// Run は流れの線の上にある。押したときに、線の高さ変更を始めない。
runButton.addEventListener("pointerdown", (event) => event.stopPropagation())
connectPreviewFullscreen(outputSection, fullscreenButton)
loadLesson(currentLesson.id, "replace")

menuButton.addEventListener("click", () => {
  const open = navigation.dataset.mobileOpen !== "true"
  setNavigationOpen(open)
})
menuCloseButton.addEventListener("click", () => {
  setNavigationOpen(false, { returnFocus: true })
})
lessonFocusButton.addEventListener("click", () => showMobilePanel("lesson"))
codeFocusButton.addEventListener("click", () => {
  showMobilePanel("code")
  editor.focus()
})
navigation.addEventListener("keydown", handleNavigationKeydown)
collapsibleNavigationQuery.addEventListener("change", syncNavigationViewport)
mobileNavigationQuery.addEventListener("change", syncNavigationViewport)
previousButton.addEventListener("click", () => moveLesson(-1))
nextButton.addEventListener("click", () => moveLesson(1))
runButton.addEventListener("click", () => void run())
openPlaygroundButton?.addEventListener("click", () => {
  window.location.assign(
    tourOpenInPlaygroundUrl(
      window.location.origin,
      localeController.current(),
      source
    )
  )
})
resetButton.addEventListener("click", () => resetLesson())
tryResetButton.addEventListener("click", () => resetLesson())
exerciseButton.addEventListener("click", () => {
  if (currentLesson.format === undefined) return
  loadSourceVariant(
    currentLesson.exerciseSource,
    localeController.messages().tourAnswerOpened,
    "Answer opened"
  )
})
diagnosticButton.addEventListener("click", () => {
  if (currentLesson.format === undefined) return
  loadSourceVariant(
    currentLesson.diagnosticSource,
    "間違いの例を開きました。エラーの知らせを読んでみてください。",
    "Mistake example opened"
  )
})
formatButton.addEventListener("click", () => void formatSource())
showTextButton.addEventListener("click", () => closeInteractivePreview())
showPreviewButton.addEventListener("click", () => {
  if (!htmlPreview.hasAttribute("srcdoc"))
    renderHtmlPreview(output.textContent ?? "")
  setOutputMode("html")
})
document.addEventListener("keydown", (event) => {
  if (!event.metaKey && !event.ctrlKey) return
  if (event.key === "Enter") {
    event.preventDefault()
    if (!runButton.disabled) void run()
    return
  }
  if (event.key.toLowerCase() === "k" && !event.shiftKey && !event.altKey) {
    event.preventDefault()
    searchButton.click()
  }
})
window.addEventListener("popstate", () => {
  const lessonId = new URL(window.location.href).searchParams.get("lesson")
  const target = findTourLesson(lessonId).id
  if (target !== currentLesson.id) loadLesson(target, "none")
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
  const directionLabel = direction === "previous" ? "← Previous" : "Next"
  if (neighbor === undefined) {
    button.textContent =
      direction === "previous"
        ? "最初のレッスンです"
        : "最後のレッスンです。お疲れさまでした。"
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
  boundary.textContent = `${direction === "previous" ? "前" : "次"}の部 · ${neighbor.categoryTitle}`
  button.replaceChildren(directionText, title, boundary)
  button.setAttribute(
    "aria-label",
    `${direction === "previous" ? "前" : "次"}のレッスン、${neighbor.title}${
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
  tryState = { ran: false, changed: false }
  furtherDetails.open = false
  showMobilePanel("lesson")
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
  showTextOutput(localeController.messages().outputPlaceholder, "idle")
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
  focusList.replaceChildren(
    ...currentLesson.focus.map((focus) => listItem(focus))
  )
  topicList.replaceChildren(
    ...currentLesson.introducedSurfaces.map((topic) => listItem(topic))
  )
  renderLessonFormat(currentLesson.format)
  stepLabel.textContent = `${currentLesson.position}/${tourLessons.length}`
  renderChapterProgress(navigationModel)
  renderTrySteps()
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
  renderChapterExamples()
  renderDeepDiveLinks()
  renderNavigation()
}

function renderChapterExamples(): void {
  if (chapterExamplesSection === null || chapterExamplesList === null) return
  if (!isChapterEnd(tourLessons, currentLesson.id)) {
    chapterExamplesSection.hidden = true
    chapterExamplesList.replaceChildren()
    return
  }
  const lessonId = currentLesson.id
  const draw = (allSamples: readonly PlaygroundSample[]): void => {
    if (currentLesson.id !== lessonId) return
    const related = chapterExamplesForLesson(tourLessons, lessonId, allSamples)
    chapterExamplesSection.hidden = related.length === 0
    const locale = localeController.current()
    const baseHref = localizedPlaygroundHref(locale)
    chapterExamplesList.replaceChildren(
      ...related.map((sample) => {
        const link = document.createElement("a")
        link.className = "tour-chapter-example-card"
        link.dataset.chapterExample = sample.id
        link.href = `${baseHref}?example=${encodeURIComponent(sample.id)}`
        const head = document.createElement("span")
        head.className = "tour-chapter-example-head"
        const title = document.createElement("strong")
        title.textContent = sample.title
        if (locale === "en") title.setAttribute("lang", "ja")
        head.append(title)
        if (locale === "en") {
          const badge = document.createElement("span")
          badge.className = "ui-tag"
          badge.textContent = JAPANESE_ONLY_LABEL
          head.append(badge)
        }
        const summary = document.createElement("small")
        summary.textContent = sample.summary
        if (locale === "en") summary.setAttribute("lang", "ja")
        link.append(head, summary)
        return link
      })
    )
  }
  if (cachedChapterSamples !== undefined) {
    draw(cachedChapterSamples)
    return
  }
  void import("../samples").then(({ samples }) => {
    cachedChapterSamples = samples
    draw(samples)
  })
}

function renderDeepDiveLinks(): void {
  const related = deepDiveArticlesForTourLesson(currentLesson.id)
  const completedTour =
    progress.completedLessonIds.length === tourLessons.length
  const articles =
    related.length > 0
      ? related
      : completedTour && currentLesson.id === tourLessons.at(-1)?.id
        ? deepDiveArticles.slice(0, 1)
        : []
  deepDiveSection.hidden = articles.length === 0
  deepDiveList.replaceChildren(
    ...articles.map((article) => {
      const item = document.createElement("li")
      const link = document.createElement("a")
      link.href = deepDiveRelativeUrl(article.id)
      link.textContent = article.title
      const summary = document.createElement("span")
      summary.textContent = article.summary
      item.append(link, summary)
      return item
    })
  )
}

function renderLessonFormat(format: TourLessonFormat | undefined): void {
  const structured = format !== undefined
  guideSection.hidden = structured
  topicList.hidden = structured
  exerciseButton.hidden = !structured
  for (const section of [
    prerequisiteSection,
    walkthroughSection,
    introducedSection,
    diagnosticSection,
    recapSection,
    nextSection,
  ]) {
    section.hidden = !structured
  }
  applyWalkthroughPins()
  expectedOutput.hidden = currentLesson.interactive
  expectedOutput.textContent =
    currentLesson.expectedFailure === ""
      ? currentLesson.expectedOutput
      : currentLesson.expectedOutput === ""
        ? `typed failure:\n${currentLesson.expectedFailure}`
        : `stdout:\n${currentLesson.expectedOutput}\n\ntyped failure:\n${currentLesson.expectedFailure}`
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

  walkthrough.replaceChildren(
    ...format.walkthrough.map((step) => walkthroughCard(step))
  )
  applyWalkthroughPins()
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
  code.className = "seseragi-highlight"
  code.append(
    ...highlightSeseragi(
      sourceExcerpt(currentLesson.source, step.sourceRange)
    ).map(({ text, classes }) => {
      const part = document.createElement("span")
      part.textContent = text
      if (classes !== "") part.className = classes
      return part
    })
  )
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
    applyWalkthroughPins()
    liveAnalysis.schedule(source)
  }
  showMobilePanel("code")
  const start = editor.state.doc.line(range.startLine)
  const end = editor.state.doc.line(range.endLine)
  editor.dispatch({
    selection: { anchor: start.from, head: end.to },
    scrollIntoView: true,
  })
  editor.focus()
}

type NavigationCloseOptions = Readonly<{
  restoreScroll?: boolean
  returnFocus?: boolean
}>

function setNavigationOpen(
  requestedOpen: boolean,
  options: NavigationCloseOptions = {}
): void {
  // コースマップは、どの画面幅でも上のバーの下に重ねて開く
  const collapsible = true
  const wasOpen = navigation.dataset.mobileOpen === "true"
  const open = collapsible && requestedOpen
  navigation.dataset.mobileOpen = String(open)
  navigation.hidden = !open
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
  for (const element of [lesson, lab]) {
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
  applyWalkthroughPins()
  showTextOutput("レッスンのコードを、最初の状態に戻しました。")
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
  applyWalkthroughPins()
  editor.dispatch(setDiagnostics(editor.state, []))
  showTextOutput(outputMessage)
  liveAnalysis.schedule(source)
  setStatus("ready", statusMessage)
  showMobilePanel("code")
  editor.focus()
}

async function formatSource(): Promise<void> {
  const requestedSource = source
  setStatus("running", "Formatting…")
  try {
    const lineWidth = resolveEditorLineWidth(
      editor,
      preferences.get().formatWidth
    )
    const formatted = await formatSingleFile(requestedSource, { lineWidth })
    if (source !== requestedSource) return
    if (formatted.status === "failure") {
      const diagnostics = formatted.diagnostics.diagnostics
      editor.dispatch(
        setDiagnostics(editor.state, [
          ...toEditorDiagnostics(requestedSource, diagnostics),
        ])
      )
      showDiagnostics(diagnostics, requestedSource)
      setStatus("error", `Cannot format: ${problemCount(diagnostics.length)}`)
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
  networkGate.reset()
  const revision = runRevision
  runButton.disabled = true
  showTextOutput("Compiling…")
  setStatus("running", "Compiling…")
  const requestedSource = source
  const requestedInput = stdinInput.value
  try {
    const compiled = await compileSingleFile(requestedSource)
    if (revision !== runRevision) return
    const diagnostics = compiled.diagnostics.diagnostics
    editor.dispatch(
      setDiagnostics(editor.state, [
        ...toEditorDiagnostics(source, diagnostics),
      ])
    )
    if (compiled.status === "failure") {
      showDiagnostics(diagnostics)
      setStatus("error", problemCount(diagnostics.length))
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
    if (revision !== runRevision) return
    runNetworkAllowed = await networkGate.resolve(
      contractNeedsNetwork(compiled.entry)
    )
    if (revision !== runRevision) return
    const execution = await prepareIsolatedExecution(
      {
        modules: [
          { path: "main.ssrg", typescript: compiled.generated.typescript },
        ],
        entryPath: "main.ssrg",
        entry: compiled.entry,
        input: requestedInput,
        storageScope: `tour-${currentLesson.id}`,
        initialHref: new URL(
          `${import.meta.env.BASE_URL}runtime-preview.html`,
          window.location.origin
        ).href,
      },
      () => {
        if (revision !== runRevision) return
        output.textContent = "Preview で動いています。"
        setOutputMode("html")
        setStatus("success", "Interactive")
        markTryProgress(undefined)
        completeCurrentLesson()
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
        if (completion.kind === "cancelled") {
          showTextOutput("Preview の実行を終えました。")
          setStatus("ready", "Preview closed")
          return
        }
        showExecutionOutput(completion.result.stdout)
        setStatus("success", "Completed")
        markTryProgress(completion.result.stdout)
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
    if (isPreviewNavigationCancellation(error)) return
    showTextOutput(error instanceof Error ? error.message : String(error))
    setStatus("error", "Execution failed")
  } finally {
    if (revision === runRevision) runButton.disabled = false
  }
}

function cancelActiveExecution(): void {
  runRevision += 1
  activePreviewPreparation?.abort()
  activePreviewPreparation = undefined
  runButton.disabled = false
  const execution = activeExecution
  activeExecution = undefined
  if (execution !== undefined) void execution.cancel()
}

function showExecutionOutput(stdout: string): void {
  clearHtmlPreview()
  output.className = "output-text"
  delete output.dataset.liveDiagnostics
  delete output.dataset.uiPlaceholder
  output.textContent = stdout || "Program completed with no output."
  if (outputMode === "html") renderHtmlPreview(stdout)
  setOutputMode(outputMode)
}

function showTextOutput(
  message: string,
  placeholder?: "idle" | "no-errors"
): void {
  runCheck.hidden = true
  output.className = "output-text"
  delete output.dataset.liveDiagnostics
  if (placeholder === undefined) delete output.dataset.uiPlaceholder
  else output.dataset.uiPlaceholder = placeholder
  output.textContent = message
  clearHtmlPreview()
  renderOutputMode("text")
}

function showDiagnostics(
  diagnostics: readonly Diagnostic[],
  analyzedSource = source
): void {
  delete output.dataset.uiPlaceholder
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
  const request = staticPreviewRequest
  void networkGate.resolve(htmlNeedsNetwork(html)).then((allowed) => {
    if (request === staticPreviewRequest) showStaticPreview(html, allowed)
  })
}

function showStaticPreview(html: string, networkAllowed: boolean): void {
  const url = URL.createObjectURL(
    new Blob([previewLinks.staticDocument(html, networkAllowed)], {
      type: "text/html",
    })
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

async function prepareIsolatedExecution(
  request: IsolatedExecutionRequest,
  onDomMounted: () => void
): Promise<BrowserExecution> {
  clearHtmlPreview()
  const preparation = new AbortController()
  activePreviewPreparation = preparation
  try {
    const execution = await startIsolatedProject(htmlPreview, request, {
      signal: preparation.signal,
      onDomMounted,
      onOpenLink: previewLinks.openLink,
      onNetworkBlocked: () => networkGate.blocked(),
      networkAllowed: runNetworkAllowed,
    })
    if (
      preparation.signal.aborted ||
      activePreviewPreparation !== preparation
    ) {
      await execution.cancel()
      throw previewNavigationCancelled()
    }
    return execution
  } finally {
    if (activePreviewPreparation === preparation)
      activePreviewPreparation = undefined
  }
}

function clearHtmlPreview(): void {
  staticPreviewRequest += 1
  activePreviewPreparation?.abort()
  activePreviewPreparation = undefined
  previewLinks.release()
  if (htmlPreviewUrl !== undefined) URL.revokeObjectURL(htmlPreviewUrl)
  htmlPreviewUrl = undefined
  htmlPreview.removeAttribute("srcdoc")
  htmlPreview.removeAttribute("src")
}

function setOutputMode(mode: "text" | "html"): void {
  outputMode = mode
  renderOutputMode(mode)
}

function closeInteractivePreview(): void {
  const wasRunning = activeExecution !== undefined
  const hasPreview =
    !htmlPreview.hidden ||
    htmlPreview.getAttribute("src") !== null ||
    htmlPreview.getAttribute("srcdoc") !== null
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

// 下の帯の状態の言葉。Output の Problems と同じ言い方にする（DESIGN.md「画面の言葉」）。
function problemCount(count: number): string {
  return `${count} ${count === 1 ? "problem" : "problems"}`
}

// 狭い画面では、レッスンとコードを下の Lesson / Code で切り替える。
function showMobilePanel(panel: "lesson" | "code"): void {
  workspace.dataset.mobilePanel = panel
  lessonFocusButton.setAttribute("aria-pressed", String(panel === "lesson"))
  codeFocusButton.setAttribute("aria-pressed", String(panel === "code"))
  if (panel === "code") requestAnimationFrame(() => editor.requestMeasure())
}

// 章の中の進み具合を、レッスンの数だけ区切ったバーで見せる。
function renderChapterProgress(
  model: ReturnType<typeof buildTourNavigationModel>
): void {
  const chapter = model.categories
    .flatMap(({ chapters }) => chapters)
    .find(({ id }) => id === currentLesson.chapterId)
  const lessons = chapter?.lessons ?? []
  chapterProgress.replaceChildren(
    ...lessons.map((item) => {
      const segment = document.createElement("span")
      segment.dataset.state = progress.completedLessonIds.includes(item.id)
        ? "done"
        : item.state === "current"
          ? "current"
          : "todo"
      return segment
    })
  )
  const done = lessons.filter((item) =>
    progress.completedLessonIds.includes(item.id)
  ).length
  chapterProgress.setAttribute(
    "aria-label",
    `この章の進み具合 ${done} / ${lessons.length}`
  )
}

// Try it: 自分で書き換えて Run する手順。終わった手順にチェックを付け、今の手順を示す。
function renderTrySteps(): void {
  const format = currentLesson.format
  const steps: { done: boolean; render: (target: HTMLElement) => void }[] = [
    {
      done: tryState.ran,
      render: (target) => {
        target.textContent = currentLesson.interactive
          ? "Run を押して、Preview で動かしてみる"
          : currentLesson.expectedFailure === ""
            ? "Run を押して、出力を確かめる"
            : "Run を押して、出力と失敗の知らせを確かめる"
      },
    },
  ]
  if (format !== undefined) {
    steps.push({
      done: tryState.changed,
      render: (target) => renderTourInline(target, format.exercise.instruction),
    })
  } else if (currentLesson.challenge !== "") {
    steps.push({
      done: tryState.changed,
      render: (target) => {
        target.textContent = currentLesson.challenge
      },
    })
  }
  const current = steps.findIndex(({ done }) => !done)
  trySteps.replaceChildren(
    ...steps.map((step, index) => {
      const item = document.createElement("li")
      item.className = "tour-try-step"
      item.dataset.state = step.done
        ? "done"
        : index === current
          ? "current"
          : "todo"
      const mark = document.createElement("span")
      mark.className = "tour-try-mark"
      mark.setAttribute("aria-hidden", "true")
      const body = document.createElement("span")
      body.className = "tour-try-body"
      step.render(body)
      if (step.done) {
        const done = document.createElement("span")
        done.className = "visually-hidden"
        done.textContent = "（済み）"
        body.append(done)
      }
      item.append(mark, body)
      return item
    })
  )
}

// Run の結果を、期待する出力と比べて Try it の手順に反映する。
// interactive なレッスンは、Preview が動いた時点で一つ目の手順を終えたとみなす。
function markTryProgress(stdout: string | undefined): void {
  const normalized = stdout?.replace(/\s+$/u, "")
  const expected = currentLesson.expectedOutput.replace(/\s+$/u, "")
  const exercise = currentLesson.exerciseExpectedOutput.replace(/\s+$/u, "")
  tryState = {
    ran: true,
    changed:
      tryState.changed ||
      (normalized !== undefined &&
        exercise !== "" &&
        exercise !== expected &&
        normalized === exercise),
  }
  runCheck.hidden =
    normalized === undefined ||
    !(normalized === expected || (exercise !== "" && normalized === exercise))
  renderTrySteps()
}

// Walkthrough の番号と同じピンを、エディターの該当する行に出す。
// 答えや間違いの例を開いているときは、行がずれるので出さない。
function applyWalkthroughPins(): void {
  const steps = currentLesson.format?.walkthrough ?? []
  setEditorLinePins(
    editor,
    source === currentLesson.source
      ? steps.map((step, index) => ({
          line: step.sourceRange.startLine,
          label: String(index + 1),
        }))
      : []
  )
}

// 外部通信の許可を覚える単位。Tour ではレッスンごと
function currentNetworkScope(): NetworkScope {
  return {
    scope: `lesson:${currentLesson.id}`,
    label: `Tour: ${currentLesson.title}`,
  }
}
