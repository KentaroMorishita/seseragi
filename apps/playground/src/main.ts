import { setDiagnostics } from "@codemirror/lint"
import { compileSingleFile } from "./compiler/wasm-driver"
import {
  renderDiagnostics,
  toEditorDiagnostics,
} from "./diagnostics/editor-diagnostics"
import { createEditor, replaceEditorSource } from "./editor/create-editor"
import { executeGeneratedModule } from "./runtime/browser-execution"
import { samples } from "./samples"
import "./styles.css"
import { requiredElement } from "./ui/elements"
import { connectMobilePanels } from "./ui/mobile-panels"

const editorHost = requiredElement("#editor", HTMLDivElement)
const sampleSelect = requiredElement("#sample-select", HTMLSelectElement)
const runButton = requiredElement("#run-button", HTMLButtonElement)
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
const statusText = requiredElement("#status-text", HTMLSpanElement)
const statusDot = requiredElement("#status-dot", HTMLSpanElement)
const workspace = requiredElement(".workspace", HTMLElement)

let source = samples[0]?.source ?? ""

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
if (initialSample) stdinInput.value = initialSample.stdin

sampleSelect.addEventListener("change", () => {
  const sample = samples.find(
    (candidate) => candidate.id === sampleSelect.value
  )
  if (!sample) return
  source = sample.source
  stdinInput.value = sample.stdin
  replaceEditorSource(editor, source)
  output.textContent = "Runを押すと結果がここに表示されます。"
  setStatus("ready", "Sample loaded")
})

runButton.addEventListener("click", () => void run())
clearSourceButton.addEventListener("click", () => {
  source = ""
  replaceEditorSource(editor, source)
  editor.dispatch(setDiagnostics(editor.state, []))
  editor.focus()
  setStatus("ready", "Source cleared")
})
clearOutputButton.addEventListener("click", () => {
  output.textContent = ""
})
connectMobilePanels(workspace)

async function run(): Promise<void> {
  runButton.disabled = true
  output.textContent = "Compiling with the shared Rust driver…"
  setStatus("running", "Compiling…")

  try {
    const compiled = await compileSingleFile(source)
    const diagnostics = compiled.diagnostics.diagnostics
    editor.dispatch(
      setDiagnostics(editor.state, [
        ...toEditorDiagnostics(source, diagnostics),
      ])
    )
    if (compiled.status === "failure") {
      output.textContent = renderDiagnostics(diagnostics)
      setStatus("error", `${diagnostics.length} diagnostic(s)`)
      showIoOnSmallScreens()
      return
    }
    if (!compiled.entry) {
      output.textContent =
        compiled.entryError ?? "Compile succeeded. No executable main found."
      setStatus("ready", "Compile succeeded")
      showIoOnSmallScreens()
      return
    }

    setStatus("running", "Running…")
    const result = await executeGeneratedModule(
      compiled.generated.typescript,
      compiled.entry,
      stdinInput.value
    )
    output.textContent = result.stdout || "Program completed with no output."
    setStatus("success", "Completed")
    showIoOnSmallScreens()
  } catch (error) {
    output.textContent = error instanceof Error ? error.message : String(error)
    setStatus("error", "Execution failed")
    showIoOnSmallScreens()
  } finally {
    runButton.disabled = false
  }
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
