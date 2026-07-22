import { setDiagnostics } from "@codemirror/lint"
import { analysisHoverAt } from "./analysis/hover"
import {
  createLiveAnalysis,
  type LiveAnalysisController,
} from "./analysis/live-analysis"
import type { AnalysisDocument, Diagnostic } from "./compiler/types"
import {
  analyzeSingleFile,
  compileSingleFile,
} from "./compiler/wasm-driver"
import { renderDiagnosticCards } from "./diagnostics/diagnostic-cards"
import { toEditorDiagnostics } from "./diagnostics/editor-diagnostics"
import { utf8RangeToUtf16 } from "./diagnostics/source-range"
import { createEditor, replaceEditorSource } from "./editor/create-editor"
import { createPreviewDocument } from "./preview-document"
import {
  type BrowserExecution,
  startGeneratedModule,
} from "./runtime/browser-execution"
import { learningPaths, samples } from "./samples"
import "./styles.css"
import { requiredElement } from "./ui/elements"
import { connectMobilePanels } from "./ui/mobile-panels"
import { connectPanelLayout } from "./ui/panel-layout"
import { connectReferenceBrowser } from "./ui/reference-browser"
import { connectSampleBrowser } from "./ui/sample-browser"
import { connectSampleGuide } from "./ui/sample-guide"

const editorHost = requiredElement("#editor", HTMLDivElement)
const sampleBrowserButton = requiredElement(
  "#sample-browser-button",
  HTMLButtonElement
)
const sampleBrowserDialog = requiredElement(
  "#sample-browser-dialog",
  HTMLDialogElement
)
const sampleBrowserClose = requiredElement(
  "#sample-browser-close",
  HTMLButtonElement
)
const referenceBrowserButton = requiredElement(
  "#reference-browser-button",
  HTMLButtonElement
)
const referenceBrowserDialog = requiredElement(
  "#reference-browser-dialog",
  HTMLDialogElement
)
const referenceBrowserClose = requiredElement(
  "#reference-browser-close",
  HTMLButtonElement
)
const referenceSearch = requiredElement(
  "#reference-search",
  HTMLInputElement
)
const referenceCategory = requiredElement(
  "#reference-category",
  HTMLSelectElement
)
const referenceResultCount = requiredElement(
  "#reference-result-count",
  HTMLElement
)
const referenceResults = requiredElement("#reference-results", HTMLElement)
const sampleBrowserLearnTab = requiredElement(
  "#sample-browser-learn-tab",
  HTMLButtonElement
)
const sampleBrowserDiscoverTab = requiredElement(
  "#sample-browser-discover-tab",
  HTMLButtonElement
)
const sampleBrowserLearnPanel = requiredElement(
  "#sample-browser-learn-panel",
  HTMLElement
)
const sampleBrowserDiscoverPanel = requiredElement(
  "#sample-browser-discover-panel",
  HTMLElement
)
const sampleLearningPaths = requiredElement(
  "#sample-learning-paths",
  HTMLElement
)
const sampleSearch = requiredElement("#sample-search", HTMLInputElement)
const sampleKindFilter = requiredElement(
  "#sample-kind-filter",
  HTMLSelectElement
)
const sampleTopicFilter = requiredElement(
  "#sample-topic-filter",
  HTMLSelectElement
)
const sampleCapabilityFilter = requiredElement(
  "#sample-capability-filter",
  HTMLSelectElement
)
const sampleFeaturedFilter = requiredElement(
  "#sample-featured-filter",
  HTMLInputElement
)
const sampleNewFilter = requiredElement("#sample-new-filter", HTMLInputElement)
const sampleResultCount = requiredElement("#sample-result-count", HTMLElement)
const sampleDiscoverResults = requiredElement(
  "#sample-discover-results",
  HTMLElement
)
const currentSampleContext = requiredElement(
  "#current-sample-context",
  HTMLElement
)
const currentSampleTitle = requiredElement("#current-sample-title", HTMLElement)
const runButton = requiredElement("#run-button", HTMLButtonElement)
const resetSampleButton = requiredElement(
  "#reset-sample-button",
  HTMLButtonElement
)
const stdinToggleButton = requiredElement(
  "#stdin-toggle-button",
  HTMLButtonElement
)
const clearSourceButton = requiredElement(
  "#clear-source-button",
  HTMLButtonElement
)
const clearOutputButton = requiredElement(
  "#clear-output-button",
  HTMLButtonElement
)
const sampleGuideButton = requiredElement(
  "#sample-guide-button",
  HTMLButtonElement
)
const sampleGuidePanel = requiredElement("#sample-guide", HTMLElement)
const sampleGuideClose = requiredElement(
  "#sample-guide-close",
  HTMLButtonElement
)
const sampleGuideCategory = requiredElement(
  "#sample-guide-category",
  HTMLElement
)
const sampleGuideTitle = requiredElement("#sample-guide-title", HTMLElement)
const sampleGuideSummary = requiredElement("#sample-guide-summary", HTMLElement)
const sampleGuideConcepts = requiredElement(
  "#sample-guide-concepts",
  HTMLUListElement
)
const sampleGuideBody = requiredElement("#sample-guide-body", HTMLElement)
const sampleGuideSource = requiredElement("#sample-guide-source", HTMLElement)
const stdinInput = requiredElement("#stdin-input", HTMLTextAreaElement)
const output = requiredElement("#output", HTMLElement)
const htmlPreview = requiredElement("#html-preview", HTMLIFrameElement)
const showTextOutputButton = requiredElement(
  "#show-text-output-button",
  HTMLButtonElement
)
const showHtmlPreviewButton = requiredElement(
  "#show-html-preview-button",
  HTMLButtonElement
)
const statusText = requiredElement("#status-text", HTMLSpanElement)
const statusDot = requiredElement("#status-dot", HTMLSpanElement)
const workspace = requiredElement(".workspace", HTMLElement)
const workspaceResizer = requiredElement("#workspace-resizer", HTMLElement)
const ioPanel = requiredElement("#io-panel", HTMLElement)
const ioResizer = requiredElement("#io-resizer", HTMLElement)
const sampleGuide = connectSampleGuide({
  button: sampleGuideButton,
  panel: sampleGuidePanel,
  closeButton: sampleGuideClose,
  category: sampleGuideCategory,
  title: sampleGuideTitle,
  summary: sampleGuideSummary,
  topics: sampleGuideConcepts,
  body: sampleGuideBody,
  source: sampleGuideSource,
})
const referenceBrowser = connectReferenceBrowser({
  button: referenceBrowserButton,
  dialog: referenceBrowserDialog,
  closeButton: referenceBrowserClose,
  search: referenceSearch,
  category: referenceCategory,
  count: referenceResultCount,
  results: referenceResults,
})

const initialSample =
  samples.find((sample) => sample.id === learningPaths[0]?.samples[0]) ??
  samples[0]
let source = initialSample?.source ?? ""
let outputMode: "text" | "html" = initialSample?.outputMode ?? "text"
let htmlPreviewUrl: string | undefined
let activeExecution: BrowserExecution | undefined
let runRevision = 0
let currentSample = initialSample
let latestAnalysis: AnalysisDocument | undefined

const sampleBrowser = connectSampleBrowser(
  {
    button: sampleBrowserButton,
    dialog: sampleBrowserDialog,
    closeButton: sampleBrowserClose,
    learnTab: sampleBrowserLearnTab,
    discoverTab: sampleBrowserDiscoverTab,
    learnPanel: sampleBrowserLearnPanel,
    discoverPanel: sampleBrowserDiscoverPanel,
    learningPaths: sampleLearningPaths,
    search: sampleSearch,
    kindFilter: sampleKindFilter,
    topicFilter: sampleTopicFilter,
    capabilityFilter: sampleCapabilityFilter,
    featuredFilter: sampleFeaturedFilter,
    newFilter: sampleNewFilter,
    resultCount: sampleResultCount,
    results: sampleDiscoverResults,
    currentContext: currentSampleContext,
    currentTitle: currentSampleTitle,
  },
  samples,
  learningPaths,
  (sample) => loadSample(sample, "Sample loaded")
)

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
    setStatus("error", error instanceof Error ? error.message : "Analysis failed")
  },
  apply: (analysis, analyzedSource) => {
    latestAnalysis = analysis
    referenceBrowser.setCatalog(analysis.standardLibrary)
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
liveAnalysis.schedule(source)

if (initialSample) {
  stdinInput.value = initialSample.stdin
  setStdinVisible(initialSample.stdin !== "")
  sampleBrowser.setCurrent(initialSample)
  sampleGuide.setSample(initialSample)
}

runButton.addEventListener("click", () => void run())
resetSampleButton.addEventListener("click", () => {
  if (!currentSample) return
  loadSample(currentSample, "Sample reset")
  editor.focus()
})
stdinToggleButton.addEventListener("click", () => {
  const visible = ioPanel.dataset.stdinCollapsed === "true"
  setStdinVisible(visible)
  if (visible) stdinInput.focus()
})
clearSourceButton.addEventListener("click", () => {
  cancelActiveExecution()
  source = ""
  replaceEditorSource(editor, source)
  editor.dispatch(setDiagnostics(editor.state, []))
  editor.focus()
  setStatus("ready", "Source cleared")
})
clearOutputButton.addEventListener("click", () => {
  cancelActiveExecution()
  output.textContent = ""
  clearHtmlPreview()
})
showTextOutputButton.addEventListener("click", () => chooseOutputMode("text"))
showHtmlPreviewButton.addEventListener("click", () => chooseOutputMode("html"))
connectMobilePanels(workspace)
connectPanelLayout({ workspace, workspaceResizer, ioPanel, ioResizer })
document.addEventListener("keydown", (event) => {
  if (event.key !== "Enter" || (!event.metaKey && !event.ctrlKey)) return
  event.preventDefault()
  if (!runButton.disabled) void run()
})

function loadSample(sample: (typeof samples)[number], status: string): void {
  cancelActiveExecution()
  currentSample = sample
  source = sample.source
  outputMode = sample.outputMode
  stdinInput.value = sample.stdin
  setStdinVisible(sample.stdin !== "")
  sampleBrowser.setCurrent(sample)
  sampleGuide.setSample(sample)
  replaceEditorSource(editor, source)
  editor.dispatch(setDiagnostics(editor.state, []))
  showTextOutput("Runを押すと結果がここに表示されます。")
  setStatus("ready", status)
}

function setStdinVisible(visible: boolean): void {
  ioPanel.dataset.stdinCollapsed = String(!visible)
  stdinToggleButton.setAttribute("aria-pressed", String(visible))
  stdinToggleButton.title = visible
    ? "標準入力パネルを隠す"
    : "標準入力パネルを表示"
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
      showIoOnSmallScreens()
      return
    }
    if (!compiled.entry) {
      showTextOutput(
        compiled.entryError ?? "Compile succeeded. No executable main found."
      )
      setStatus("ready", "Compile succeeded")
      showIoOnSmallScreens()
      return
    }

    setStatus("running", "Running…")
    const needsDom = compiled.entry.environment.some(
      (binding) => binding.service === "dom"
    )
    const domDocument = needsDom ? await prepareInteractivePreview() : undefined
    if (revision !== runRevision) {
      clearHtmlPreview()
      setOutputMode("text")
      return
    }
    const execution = await startGeneratedModule(
      compiled.generated.typescript,
      compiled.entry,
      stdinInput.value,
      {
        ...(domDocument === undefined ? {} : { domDocument }),
        onDomMounted: () => {
          if (revision !== runRevision) return
          output.className = "output-text"
          output.textContent = "Interactive preview is running."
          setOutputMode("html")
          setStatus("success", "Interactive")
          showIoOnSmallScreens()
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
        showIoOnSmallScreens()
      },
      (error: unknown) => {
        if (revision !== runRevision || activeExecution !== execution) return
        activeExecution = undefined
        showTextOutput(error instanceof Error ? error.message : String(error))
        setStatus("error", "Execution failed")
        showIoOnSmallScreens()
      }
    )
  } catch (error) {
    if (revision !== runRevision) return
    showTextOutput(error instanceof Error ? error.message : String(error))
    setStatus("error", "Execution failed")
    showIoOnSmallScreens()
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
  setOutputMode(outputMode)
  renderHtmlPreview(stdout)
}

function showTextOutput(message: string): void {
  output.className = "output-text"
  delete output.dataset.liveDiagnostics
  output.textContent = message
  clearHtmlPreview()
  setOutputMode("text")
}

function showDiagnostics(
  diagnostics: readonly Diagnostic[],
  analyzedSource = source
): void {
  clearHtmlPreview()
  setOutputMode("text")
  output.dataset.liveDiagnostics = "true"
  renderDiagnosticCards(output, diagnostics, (byteRange) => {
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
  const document = htmlPreview.contentDocument
  if (document === null) {
    throw new Error("interactive preview document is unavailable")
  }
  setOutputMode("html")
  return document
}

function clearHtmlPreview(): void {
  if (htmlPreviewUrl !== undefined) URL.revokeObjectURL(htmlPreviewUrl)
  htmlPreviewUrl = undefined
  htmlPreview.removeAttribute("src")
}

function chooseOutputMode(mode: "text" | "html"): void {
  outputMode = mode
  setOutputMode(mode)
}

function setOutputMode(mode: "text" | "html"): void {
  const showHtml = mode === "html"
  output.hidden = showHtml
  htmlPreview.hidden = !showHtml
  showTextOutputButton.setAttribute("aria-pressed", String(!showHtml))
  showHtmlPreviewButton.setAttribute("aria-pressed", String(showHtml))
}

function setStatus(
  state: "ready" | "running" | "success" | "error",
  message: string
): void {
  statusDot.dataset.state = state
  statusText.textContent = message
}

function showIoOnSmallScreens(): void {
  if (!window.matchMedia("(max-width: 760px)").matches) return
  const tab = document.querySelector<HTMLButtonElement>(
    '[data-panel-target="io"]'
  )
  tab?.click()
}
