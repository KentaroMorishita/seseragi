import { setDiagnostics } from "@codemirror/lint"
import { analysisHoverAt } from "./analysis/hover"
import {
  createLiveAnalysis,
  type LiveAnalysisController,
} from "./analysis/live-analysis"
import type {
  AnalysisDocument,
  ProjectAnalysisResponse,
  ProjectRequest,
} from "./compiler/types"
import {
  analyzeProject,
  compileProject,
  formatProjectFile,
} from "./compiler/wasm-driver"
import { renderWorkspaceDiagnosticCards } from "./diagnostics/diagnostic-cards"
import { toEditorDiagnostics } from "./diagnostics/editor-diagnostics"
import { utf8RangeToUtf16 } from "./diagnostics/source-range"
import {
  collectWorkspaceDiagnostics,
  type WorkspaceDiagnostic,
} from "./diagnostics/workspace-diagnostics"
import {
  createEditor,
  createEditorState,
  replaceEditorSource,
  setEditorEditable,
  setEditorWhitespaceVisible,
} from "./editor/create-editor"
import { createPreviewDocument } from "./preview-document"
import {
  type BrowserExecution,
  startGeneratedProject,
} from "./runtime/browser-execution"
import { discoverGroups, samples } from "./samples"
import "./styles.css"
import { requiredElement } from "./ui/elements"
import { connectMobilePanels } from "./ui/mobile-panels"
import { connectOverflowMenu } from "./ui/overflow-menu"
import { connectPanelLayout } from "./ui/panel-layout"
import { connectPreviewFullscreen } from "./ui/preview-fullscreen"
import { connectReferenceBrowser } from "./ui/reference-browser"
import { connectSampleBrowser } from "./ui/sample-browser"
import { connectSampleGuide } from "./ui/sample-guide"
import { createWorkspaceEditorSessions } from "./workspace/editor-session"
import {
  connectWorkspaceTabs,
  type WorkspaceTabChange,
} from "./workspace/editor-tabs"
import {
  connectWorkspaceExplorer,
  readExplorerWidth,
  type WorkspaceExplorerChange,
} from "./workspace/explorer"
import {
  activateWorkspaceFile,
  activeWorkspaceSource,
  createWorkspace,
  setWorkspaceExplorer,
  updateActiveWorkspaceSource,
  type WorkspaceState,
} from "./workspace/model"
import { connectWorkspaceFocusNavigation } from "./workspace/focus-navigation"
import {
  confirmDirtySampleSwitch,
  persistWorkspace,
  restoreWorkspace,
} from "./workspace/persistence"
import {
  runnableWorkspaceProjectRequest,
  type WorkspaceAnalysisRequest,
  workspaceAnalysisRevision,
  workspaceProjectRequest,
  workspaceProjectRevision,
} from "./workspace/project-request"

type WorkspaceAnalysisResult = Readonly<{
  activeDocument?: AnalysisDocument
  diagnostics: readonly WorkspaceDiagnostic[]
}>

const editorHost = requiredElement("#editor", HTMLDivElement)
const editorSurface = requiredElement("#editor-surface", HTMLElement)
const workspaceNotice = requiredElement("#workspace-notice", HTMLElement)
const workspaceNoticeText = requiredElement(
  "#workspace-notice-text",
  HTMLElement
)
const workspaceNoticeAction = requiredElement(
  "#workspace-notice-action",
  HTMLButtonElement
)
const workspaceEmptyState = requiredElement(
  "#workspace-empty-state",
  HTMLElement
)
const workspaceEmptyTitle = requiredElement(
  "#workspace-empty-title",
  HTMLElement
)
const workspaceEmptyCopy = requiredElement("#workspace-empty-copy", HTMLElement)
const workspaceEmptyAction = requiredElement(
  "#workspace-empty-action",
  HTMLButtonElement
)
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
const mobileReferenceButton = requiredElement(
  "#mobile-reference-button",
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
const referenceSearch = requiredElement("#reference-search", HTMLInputElement)
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
const mobileResetButton = requiredElement(
  "#mobile-reset-button",
  HTMLButtonElement
)
const formatSourceButton = requiredElement(
  "#format-source-button",
  HTMLButtonElement
)
const mobileFormatButton = requiredElement(
  "#mobile-format-button",
  HTMLButtonElement
)
const whitespaceToggleButton = requiredElement(
  "#whitespace-toggle-button",
  HTMLButtonElement
)
const mobileWhitespaceButton = requiredElement(
  "#mobile-whitespace-button",
  HTMLButtonElement
)
const mobileToolsButton = requiredElement(
  "#mobile-tools-button",
  HTMLButtonElement
)
const mobileToolsMenu = requiredElement("#mobile-tools-menu", HTMLElement)
const explorerToggleButton = requiredElement(
  "#explorer-toggle-button",
  HTMLButtonElement
)
const mobileExplorerButton = requiredElement(
  "#mobile-explorer-button",
  HTMLButtonElement
)
const codeWorkspace = requiredElement("#code-workspace", HTMLElement)
const explorerPanel = requiredElement("#explorer-panel", HTMLElement)
const explorerTree = requiredElement("#explorer-tree", HTMLElement)
const explorerMessage = requiredElement("#explorer-message", HTMLElement)
const explorerResizer = requiredElement("#explorer-resizer", HTMLElement)
const explorerNewFile = requiredElement("#explorer-new-file", HTMLButtonElement)
const explorerNewFolder = requiredElement(
  "#explorer-new-folder",
  HTMLButtonElement
)
const explorerCollapseAll = requiredElement(
  "#explorer-collapse-all",
  HTMLButtonElement
)
const explorerClose = requiredElement("#explorer-close", HTMLButtonElement)
const activeFileName = requiredElement("#active-file-name", HTMLElement)
const workspaceTabs = requiredElement("#workspace-tabs", HTMLElement)
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
const fullscreenPreviewButton = requiredElement(
  "#fullscreen-preview-button",
  HTMLButtonElement
)
const statusText = requiredElement("#status-text", HTMLSpanElement)
const statusDot = requiredElement("#status-dot", HTMLSpanElement)
const workspace = requiredElement(".workspace", HTMLElement)
const workspaceResizer = requiredElement("#workspace-resizer", HTMLElement)
const ioPanel = requiredElement("#io-panel", HTMLElement)
const outputSection = requiredElement("#output-section", HTMLElement)
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
  buttons: [referenceBrowserButton, mobileReferenceButton],
  dialog: referenceBrowserDialog,
  closeButton: referenceBrowserClose,
  search: referenceSearch,
  category: referenceCategory,
  count: referenceResultCount,
  results: referenceResults,
})
connectPreviewFullscreen(outputSection, fullscreenPreviewButton)

const defaultSample =
  samples.find((sample) => sample.id === "hello-world") ?? samples[0]
const restoredWorkspace = restoreWorkspace(localStorage, samples)
const restoredSample =
  restoredWorkspace.status === "restored"
    ? samples.find(({ id }) => id === restoredWorkspace.sampleId)
    : undefined
const initialSample = restoredSample ?? defaultSample
let workspaceState =
  restoredWorkspace.status === "restored" && restoredSample !== undefined
    ? restoredWorkspace.workspace
    : setWorkspaceExplorer(
        createWorkspace(initialSample?.workspace ?? { files: [] }),
        { width: readExplorerWidth(localStorage) }
      )
let applyingWorkspaceSource = false
let outputMode: "text" | "html" = initialSample?.outputMode ?? "text"
let htmlPreviewUrl: string | undefined
let activeExecution: BrowserExecution | undefined
let runRevision = 0
let currentSample = initialSample
let latestAnalysis: AnalysisDocument | undefined
let persistenceFailureShown = false
let editorEditable: boolean | undefined

const sampleBrowser = connectSampleBrowser(
  {
    button: sampleBrowserButton,
    dialog: sampleBrowserDialog,
    closeButton: sampleBrowserClose,
    learnTab: sampleBrowserLearnTab,
    discoverTab: sampleBrowserDiscoverTab,
    learnPanel: sampleBrowserLearnPanel,
    discoverPanel: sampleBrowserDiscoverPanel,
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
  discoverGroups,
  (sample) => loadSample(sample, "Sample loaded")
)

const editor = createEditor(
  editorHost,
  activeWorkspaceSource(workspaceState),
  handleEditorChange,
  editorHoverAt
)
const editorSessions = createWorkspaceEditorSessions(editor, (source) =>
  createEditorState(source, handleEditorChange, editorHoverAt)
)
const whitespaceStorageKey = "seseragi.playground.showWhitespace"
let showWhitespace = localStorage.getItem(whitespaceStorageKey) === "true"
setWhitespaceVisible(showWhitespace)

connectOverflowMenu({ button: mobileToolsButton, menu: mobileToolsMenu })

const liveAnalysis: LiveAnalysisController =
  createLiveAnalysis<WorkspaceAnalysisResult>({
    analyze: async (_source, identity) => {
      if (identity === undefined) {
        throw new Error("Workspace analysis request is missing")
      }
      const request = JSON.parse(identity) as WorkspaceAnalysisRequest
      return workspaceAnalysisResult(
        request,
        await analyzeProject(request.project)
      )
    },
    onPending: () => {
      if (!runButton.disabled) setStatus("running", "Analyzing…")
    },
    onError: (error, _source, identity) => {
      if (!workspaceAnalysisIsCurrent(identity)) return
      if (runButton.disabled) return
      setStatus(
        "error",
        error instanceof Error ? error.message : "Analysis failed"
      )
    },
    apply: (analysis, _analyzedSource, identity) => {
      if (!workspaceAnalysisIsCurrent(identity)) return
      latestAnalysis = analysis.activeDocument
      if (analysis.activeDocument !== undefined) {
        referenceBrowser.setCatalog(analysis.activeDocument.standardLibrary)
      }
      setActiveEditorDiagnostics(analysis.diagnostics)
      if (runButton.disabled) return
      if (analysis.diagnostics.length > 0) {
        showWorkspaceDiagnostics(analysis.diagnostics)
        setStatus("error", `${analysis.diagnostics.length} diagnostic(s)`)
      } else {
        if (output.dataset.liveDiagnostics === "true") {
          showTextOutput("No diagnostics. Runでprogramを実行できます。")
        }
        setStatus("ready", "Analysis ready")
      }
    },
  })
scheduleWorkspaceAnalysis()

const tabs = connectWorkspaceTabs(workspaceTabs, {
  getState: () => workspaceState,
  onChange: applyWorkspaceChange,
  panel: editorSurface,
})

const explorer = connectWorkspaceExplorer(
  {
    codeWorkspace,
    panel: explorerPanel,
    tree: explorerTree,
    message: explorerMessage,
    resizer: explorerResizer,
    toggleButtons: [explorerToggleButton, mobileExplorerButton],
    newFileButton: explorerNewFile,
    newFolderButton: explorerNewFolder,
    collapseAllButton: explorerCollapseAll,
    closeButton: explorerClose,
  },
  {
    getState: () => workspaceState,
    onChange: applyWorkspaceChange,
  }
)

workspaceNoticeAction.addEventListener("click", explorer.focus)
workspaceEmptyAction.addEventListener("click", () => {
  if (workspaceState.files.length === 0) {
    explorer.show()
    explorerNewFile.click()
    return
  }
  explorer.focus()
})

renderWorkspaceChrome()
if (initialSample) {
  stdinInput.value =
    restoredWorkspace.status === "restored"
      ? restoredWorkspace.stdin
      : initialSample.stdin
  setStdinVisible(initialSample.stdin !== "")
  sampleBrowser.setCurrent(initialSample)
  sampleGuide.setSample(initialSample)
}
if (restoredWorkspace.status === "recovered") {
  showTextOutput(restoredWorkspace.diagnostic)
  setStatus("ready", "Workspace recovered")
}

runButton.addEventListener("click", () => void run())
const resetSample = (): void => {
  if (!currentSample) return
  loadSample(currentSample, "Workspace reset", false)
  editor.focus()
}
resetSampleButton.addEventListener("click", resetSample)
mobileResetButton.addEventListener("click", resetSample)
const formatSource = async (): Promise<void> => {
  const requestedFile = workspaceState.activeFile
  if (requestedFile === undefined) {
    setStatus("error", "Select a file before Format")
    return
  }
  const request = workspaceProjectRequest(workspaceState)
  const requestedRevision = workspaceAnalysisRevision(workspaceState)
  setStatus("running", "Formatting…")
  try {
    const formatted = await formatProjectFile(request, requestedFile)
    if (!workspaceAnalysisIsCurrent(requestedRevision)) return
    if (formatted.status === "failure") {
      const diagnostics = collectWorkspaceDiagnostics(
        request,
        formatted.diagnostics,
        formatted.problems
      )
      setActiveEditorDiagnostics(diagnostics)
      showWorkspaceDiagnostics(diagnostics)
      setStatus("error", `Cannot format: ${diagnostics.length} diagnostic(s)`)
      return
    }
    if (!formatted.changed) {
      setStatus("success", "Already formatted")
      return
    }
    workspaceState = updateActiveWorkspaceSource(
      workspaceState,
      formatted.source
    )
    replaceEditorFromWorkspace(formatted.source)
    renderWorkspaceChrome()
    scheduleWorkspaceAnalysis()
    setStatus("success", "Formatted")
    persistCurrentWorkspace()
  } catch (error) {
    if (!workspaceAnalysisIsCurrent(requestedRevision)) return
    setStatus(
      "error",
      error instanceof Error ? error.message : "Formatting failed"
    )
  } finally {
    editor.focus()
  }
}
formatSourceButton.addEventListener("click", () => void formatSource())
mobileFormatButton.addEventListener("click", () => void formatSource())
const toggleWhitespace = (): void => setWhitespaceVisible(!showWhitespace)
whitespaceToggleButton.addEventListener("click", toggleWhitespace)
mobileWhitespaceButton.addEventListener("click", toggleWhitespace)
stdinToggleButton.addEventListener("click", () => {
  const visible = ioPanel.dataset.stdinCollapsed === "true"
  setStdinVisible(visible)
  if (visible) stdinInput.focus()
})
clearSourceButton.addEventListener("click", () => {
  if (workspaceState.activeFile === undefined) {
    setStatus("error", "Open a file before clearing source")
    explorer.focus()
    return
  }
  cancelActiveExecution()
  workspaceState = updateActiveWorkspaceSource(workspaceState, "")
  replaceEditorFromWorkspace("")
  editor.dispatch(setDiagnostics(editor.state, []))
  editor.focus()
  setStatus("ready", "Source cleared")
  persistCurrentWorkspace()
})
stdinInput.addEventListener("input", persistCurrentWorkspace)
clearOutputButton.addEventListener("click", () => {
  cancelActiveExecution()
  output.textContent = ""
  clearHtmlPreview()
})
showTextOutputButton.addEventListener("click", () => chooseOutputMode("text"))
showHtmlPreviewButton.addEventListener("click", () => chooseOutputMode("html"))
const mobilePanels = connectMobilePanels(workspace)
connectPanelLayout({ workspace, workspaceResizer, ioPanel, ioResizer })
connectWorkspaceFocusNavigation(
  {
    document,
    explorerPanel,
    workspaceTabs,
    editorSurface,
    ioPanel,
  },
  {
    focusExplorer: explorer.focus,
    focusEditor: () => {
      if (workspaceState.activeFile === undefined) {
        workspaceEmptyAction.focus()
      } else {
        editor.focus()
      }
    },
    showCode: () => mobilePanels.show("code"),
    showIo: () => mobilePanels.show("io"),
  }
)
document.addEventListener("keydown", (event) => {
  if (event.key !== "Enter" || (!event.metaKey && !event.ctrlKey)) return
  event.preventDefault()
  if (!runButton.disabled) void run()
})

function loadSample(
  sample: (typeof samples)[number],
  status: string,
  confirmDirty = true
): boolean {
  if (
    confirmDirty &&
    currentSample?.id !== sample.id &&
    !confirmDirtySampleSwitch(workspaceState, sample.title, (message) =>
      window.confirm(message)
    )
  ) {
    setStatus("ready", "Sample switch canceled")
    return false
  }
  cancelActiveExecution()
  currentSample = sample
  workspaceState = createWorkspace(sample.workspace)
  outputMode = sample.outputMode
  stdinInput.value = sample.stdin
  setStdinVisible(sample.stdin !== "")
  sampleBrowser.setCurrent(sample)
  sampleGuide.setSample(sample)
  applyingWorkspaceSource = true
  try {
    editorSessions.reset(workspaceState)
  } finally {
    applyingWorkspaceSource = false
  }
  setEditorWhitespaceVisible(editor, showWhitespace)
  renderWorkspaceChrome()
  editor.dispatch(setDiagnostics(editor.state, []))
  latestAnalysis = undefined
  liveAnalysis.cancel()
  scheduleWorkspaceAnalysis()
  showTextOutput("Runを押すと結果がここに表示されます。")
  setStatus("ready", status)
  persistCurrentWorkspace()
  return true
}

function applyWorkspaceChange(
  nextState: WorkspaceState,
  change: WorkspaceExplorerChange | WorkspaceTabChange
): void {
  const previous = workspaceState
  const previousProjectRevision = workspaceProjectRevisionOrUndefined(previous)
  const previousAnalysisRevision =
    workspaceAnalysisRevisionOrUndefined(previous)
  workspaceState = nextState
  applyingWorkspaceSource = true
  let switched = false
  try {
    switched = editorSessions.transition(
      previous,
      nextState,
      "rename" in change ? change.rename : undefined
    )
  } finally {
    applyingWorkspaceSource = false
  }
  renderWorkspaceChrome()
  const projectChanged =
    previousProjectRevision !== workspaceProjectRevisionOrUndefined(nextState)
  const analysisChanged =
    previousAnalysisRevision !== workspaceAnalysisRevisionOrUndefined(nextState)
  if (projectChanged) cancelActiveExecution()
  if (switched) {
    setEditorWhitespaceVisible(editor, showWhitespace)
    editor.dispatch(setDiagnostics(editor.state, []))
    latestAnalysis = undefined
  }
  if (analysisChanged) {
    liveAnalysis.cancel()
    scheduleWorkspaceAnalysis()
  }
  if (change.message !== undefined) setStatus("ready", change.message)
  if (change.focusEditor) editor.focus()
  persistCurrentWorkspace()
}

function handleEditorChange(nextSource: string): void {
  if (!applyingWorkspaceSource) {
    if (workspaceState.activeFile === undefined) return
    const wasDirty =
      workspaceState.activeFile !== undefined &&
      workspaceState.dirtyFiles.includes(workspaceState.activeFile)
    workspaceState = updateActiveWorkspaceSource(workspaceState, nextSource)
    if (!wasDirty) renderWorkspaceChrome()
    persistCurrentWorkspace()
  }
  latestAnalysis = undefined
  editor.dispatch(setDiagnostics(editor.state, []))
  scheduleWorkspaceAnalysis()
}

function persistCurrentWorkspace(): void {
  if (currentSample === undefined) return
  const result = persistWorkspace(
    localStorage,
    currentSample,
    workspaceState,
    stdinInput.value
  )
  if (result.status === "saved") {
    persistenceFailureShown = false
    return
  }
  if (persistenceFailureShown) return
  persistenceFailureShown = true
  showTextOutput(result.diagnostic)
  setStatus("error", "Local save unavailable")
}

function editorHoverAt(position: number) {
  return analysisHoverAt(
    latestAnalysis,
    activeWorkspaceSource(workspaceState),
    position
  )
}

function renderWorkspaceChrome(): void {
  explorer.render(workspaceState)
  tabs.render(workspaceState)
  const path = workspaceState.activeFile
  const hasActiveFile = path !== undefined
  const emptyWorkspace = workspaceState.files.length === 0
  activeFileName.textContent = path ?? "No active file"
  activeFileName.dataset.dirty = String(
    hasActiveFile && workspaceState.dirtyFiles.includes(path)
  )
  editorSurface.dataset.workspaceState = emptyWorkspace
    ? "empty"
    : hasActiveFile
      ? "active"
      : "no-active-file"
  workspaceEmptyState.hidden = hasActiveFile
  workspaceEmptyTitle.textContent = emptyWorkspace
    ? "Workspace is empty"
    : "No file is open"
  workspaceEmptyCopy.textContent = emptyWorkspace
    ? "Create a Seseragi file to start editing."
    : "Choose a file from Explorer to continue editing."
  workspaceEmptyAction.textContent = emptyWorkspace
    ? "New File"
    : "Open Explorer"
  const missingEntry = hasActiveFile && workspaceState.entryFile === undefined
  workspaceNotice.hidden = !missingEntry
  workspaceNoticeText.textContent = missingEntry
    ? "No entry file. Choose Set as entry in Explorer before Run."
    : ""
  editorHost.inert = !hasActiveFile
  if (editorEditable !== hasActiveFile) {
    setEditorEditable(editor, hasActiveFile)
    editorEditable = hasActiveFile
  }
  clearSourceButton.disabled = !hasActiveFile
  formatSourceButton.disabled = !hasActiveFile
  mobileFormatButton.disabled = !hasActiveFile
}

function replaceEditorFromWorkspace(nextSource: string): void {
  applyingWorkspaceSource = true
  try {
    replaceEditorSource(editor, nextSource)
  } finally {
    applyingWorkspaceSource = false
  }
}

function workspaceAnalysisResult(
  request: WorkspaceAnalysisRequest,
  response: ProjectAnalysisResponse
): WorkspaceAnalysisResult {
  if (response.status === "failure") {
    return {
      diagnostics: collectWorkspaceDiagnostics(
        request.project,
        response.diagnostics,
        response.problems
      ),
    }
  }
  const sources = new Map(
    request.project.files.map(({ path, source }) => [path, source])
  )
  const diagnostics = response.documents.flatMap(({ path, document }) =>
    document.diagnostics.diagnostics.map((diagnostic) => ({
      path,
      source: sources.get(path) ?? "",
      diagnostic,
    }))
  )
  const activeDocument = response.documents.find(
    ({ path }) => path === request.active
  )?.document
  return {
    ...(activeDocument === undefined ? {} : { activeDocument }),
    diagnostics,
  }
}

function scheduleWorkspaceAnalysis(): void {
  if (
    workspaceState.activeFile === undefined ||
    workspaceState.files.length === 0
  ) {
    liveAnalysis.cancel()
    latestAnalysis = undefined
    editor.dispatch(setDiagnostics(editor.state, []))
    return
  }
  liveAnalysis.schedule(
    activeWorkspaceSource(workspaceState),
    workspaceAnalysisRevision(workspaceState)
  )
}

function workspaceAnalysisIsCurrent(identity: string | undefined): boolean {
  return (
    identity !== undefined &&
    workspaceAnalysisRevisionOrUndefined(workspaceState) === identity
  )
}

function workspaceProjectIsCurrent(revision: string): boolean {
  return workspaceProjectRevisionOrUndefined(workspaceState) === revision
}

function workspaceProjectRevisionOrUndefined(
  state: WorkspaceState
): string | undefined {
  try {
    return workspaceProjectRevision(state)
  } catch {
    return undefined
  }
}

function workspaceAnalysisRevisionOrUndefined(
  state: WorkspaceState
): string | undefined {
  try {
    return workspaceAnalysisRevision(state)
  } catch {
    return undefined
  }
}

function setActiveEditorDiagnostics(
  diagnostics: readonly WorkspaceDiagnostic[]
): void {
  const activeFile = workspaceState.activeFile
  if (activeFile === undefined) {
    editor.dispatch(setDiagnostics(editor.state, []))
    return
  }
  const activeSource = activeWorkspaceSource(workspaceState)
  editor.dispatch(
    setDiagnostics(
      editor.state,
      toEditorDiagnostics(
        activeSource,
        diagnostics
          .filter(({ path }) => path === activeFile)
          .map(({ diagnostic }) => diagnostic)
      )
    )
  )
}

function setStdinVisible(visible: boolean): void {
  ioPanel.dataset.stdinCollapsed = String(!visible)
  stdinToggleButton.setAttribute("aria-expanded", String(visible))
  stdinToggleButton.title = visible ? "Hide Input" : "Show Input"
}

function setWhitespaceVisible(visible: boolean): void {
  showWhitespace = visible
  setEditorWhitespaceVisible(editor, visible)
  whitespaceToggleButton.setAttribute("aria-pressed", String(visible))
  mobileWhitespaceButton.setAttribute("aria-checked", String(visible))
  localStorage.setItem(whitespaceStorageKey, String(visible))
}

async function run(): Promise<void> {
  cancelActiveExecution()
  const revision = runRevision
  let request: ProjectRequest
  try {
    request = runnableWorkspaceProjectRequest(workspaceState)
  } catch (error) {
    showTextOutput(error instanceof Error ? error.message : String(error))
    setStatus("error", "Entry required")
    showIoOnSmallScreens()
    return
  }
  const requestedRevision = JSON.stringify(request)
  runButton.disabled = true
  showTextOutput("Compiling with the shared Rust driver…")
  setStatus("running", "Compiling…")

  try {
    const compiled = await compileProject(request)
    if (revision !== runRevision) return
    if (!workspaceProjectIsCurrent(requestedRevision)) {
      setStatus("ready", "Project changed")
      return
    }
    const diagnostics = collectWorkspaceDiagnostics(
      request,
      compiled.diagnostics,
      compiled.status === "failure" ? compiled.problems : []
    )
    setActiveEditorDiagnostics(diagnostics)
    if (compiled.status === "failure") {
      showWorkspaceDiagnostics(diagnostics)
      setStatus("error", `${diagnostics.length} diagnostic(s)`)
      showIoOnSmallScreens()
      return
    }
    if (!compiled.entry.contract) {
      showTextOutput(
        compiled.entry.error ?? "Compile succeeded. No executable main found."
      )
      setStatus("ready", "Compile succeeded")
      showIoOnSmallScreens()
      return
    }

    setStatus("running", "Running…")
    const needsDom = compiled.entry.contract.environment.some(
      (binding) => binding.service === "dom"
    )
    const domDocument = needsDom ? await prepareInteractivePreview() : undefined
    if (revision !== runRevision) {
      clearHtmlPreview()
      setOutputMode("text")
      return
    }
    if (!workspaceProjectIsCurrent(requestedRevision)) {
      clearHtmlPreview()
      setOutputMode("text")
      setStatus("ready", "Project changed")
      return
    }
    const execution = await startGeneratedProject(
      compiled.modules.map(({ path, generated }) => ({
        path,
        typescript: generated.typescript,
      })),
      compiled.entry.path,
      compiled.entry.contract,
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
    if (!workspaceProjectIsCurrent(requestedRevision)) {
      setStatus("ready", "Project changed")
      return
    }
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

function showWorkspaceDiagnostics(
  diagnostics: readonly WorkspaceDiagnostic[]
): void {
  clearHtmlPreview()
  setOutputMode("text")
  output.dataset.liveDiagnostics = "true"
  renderWorkspaceDiagnosticCards(output, diagnostics, (path, byteRange) => {
    if (!workspaceState.files.some((file) => file.path === path)) return
    if (workspaceState.activeFile !== path) {
      applyWorkspaceChange(activateWorkspaceFile(workspaceState, path), {
        message: `Opened diagnostic: ${path}`,
      })
    }
    const range = utf8RangeToUtf16(
      workspaceState.files.find((file) => file.path === path)?.source ?? "",
      byteRange
    )
    editor.dispatch({
      selection: { anchor: range.from, head: range.to },
      scrollIntoView: true,
    })
    mobilePanels.show("code")
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
  if (
    !window.matchMedia(
      "(max-width: 760px), (max-width: 960px) and (max-height: 520px)"
    ).matches
  ) {
    return
  }
  const tab = document.querySelector<HTMLButtonElement>(
    '[data-panel-target="io"]'
  )
  tab?.click()
}
