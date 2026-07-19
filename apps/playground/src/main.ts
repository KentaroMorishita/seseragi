import { setDiagnostics } from "@codemirror/lint"
import { compileSingleFile } from "./compiler/wasm-driver"
import {
  renderDiagnostics,
  toEditorDiagnostics,
} from "./diagnostics/editor-diagnostics"
import { createEditor, replaceEditorSource } from "./editor/create-editor"
import { createPreviewDocument } from "./preview-document"
import {
  type BrowserExecution,
  startGeneratedModule,
} from "./runtime/browser-execution"
import { samples } from "./samples"
import "./styles.css"
import { requiredElement } from "./ui/elements"
import { connectMobilePanels } from "./ui/mobile-panels"
import { connectPanelLayout } from "./ui/panel-layout"

const editorHost = requiredElement("#editor", HTMLDivElement)
const sampleSelect = requiredElement("#sample-select", HTMLSelectElement)
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
const stdinInput = requiredElement("#stdin-input", HTMLTextAreaElement)
const output = requiredElement("#output", HTMLPreElement)
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

let source = samples[0]?.source ?? ""
let outputMode: "text" | "html" = samples[0]?.outputMode ?? "text"
let htmlPreviewUrl: string | undefined
let activeExecution: BrowserExecution | undefined
let runRevision = 0

const sampleGroups = new Map<string, HTMLOptGroupElement>()
for (const sample of samples) {
  let group = sampleGroups.get(sample.category)
  if (!group) {
    group = document.createElement("optgroup")
    group.label = sample.category
    sampleGroups.set(sample.category, group)
    sampleSelect.append(group)
  }
  const option = document.createElement("option")
  option.value = sample.id
  option.textContent = sample.label
  group.append(option)
}

const editor = createEditor(editorHost, source, (nextSource) => {
  source = nextSource
  editor.dispatch(setDiagnostics(editor.state, []))
  setStatus("ready", "Ready to run")
})

const initialSample = samples[0]
if (initialSample) {
  stdinInput.value = initialSample.stdin
  setStdinVisible(initialSample.stdin !== "")
}

sampleSelect.addEventListener("change", () => {
  const sample = samples.find(
    (candidate) => candidate.id === sampleSelect.value
  )
  if (!sample) return
  loadSample(sample, "Sample loaded")
})

runButton.addEventListener("click", () => void run())
resetSampleButton.addEventListener("click", () => {
  const sample = samples.find(
    (candidate) => candidate.id === sampleSelect.value
  )
  if (!sample) return
  loadSample(sample, "Sample reset")
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
  source = sample.source
  outputMode = sample.outputMode
  stdinInput.value = sample.stdin
  setStdinVisible(sample.stdin !== "")
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
      showTextOutput(renderDiagnostics(diagnostics))
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
  output.textContent = stdout || "Program completed with no output."
  setOutputMode(outputMode)
  renderHtmlPreview(stdout)
}

function showTextOutput(message: string): void {
  output.textContent = message
  clearHtmlPreview()
  setOutputMode("text")
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
