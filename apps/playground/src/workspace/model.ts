export type WorkspacePath = string

export type WorkspaceSourceIdentity = Readonly<{
  path: WorkspacePath
  module: string
}>

export type WorkspaceFile = Readonly<{
  path: WorkspacePath
  source: string
}>

export type WorkspaceExplorerState = Readonly<{
  visible: boolean
  width: number
}>

export type WorkspaceState = Readonly<{
  files: readonly WorkspaceFile[]
  folders: readonly WorkspacePath[]
  entryFile?: WorkspacePath
  entryModule?: string
  activeFile?: WorkspacePath
  openFiles: readonly WorkspacePath[]
  dirtyFiles: readonly WorkspacePath[]
  expandedFolders: readonly WorkspacePath[]
  explorer: WorkspaceExplorerState
}>

export type WorkspaceSeed = Readonly<{
  files: readonly Readonly<{ path: string; source: string }>[]
  folders?: readonly string[]
  entryFile?: string
  activeFile?: string
  openFiles?: readonly string[]
  dirtyFiles?: readonly string[]
  expandedFolders?: readonly string[]
  explorer?: Partial<WorkspaceExplorerState>
}>

export const defaultExplorerWidth = 240
export const minimumExplorerWidth = 180
export const maximumExplorerWidth = 480

export function createSingleFileWorkspace(source: string): WorkspaceState {
  return createWorkspace({
    files: [{ path: "main.ssrg", source }],
    entryFile: "main.ssrg",
    activeFile: "main.ssrg",
    openFiles: ["main.ssrg"],
  })
}

export function createWorkspace(seed: WorkspaceSeed): WorkspaceState {
  const files = seed.files.map(({ path, source }) => ({
    path: workspaceSourcePath(path),
    source,
  }))
  assertUniquePaths(
    "workspace file",
    files.map(({ path }) => path)
  )

  const explicitFolders = (seed.folders ?? []).map(workspacePath)
  assertUniquePaths("workspace folder", explicitFolders)
  const folders = new Set<WorkspacePath>()
  for (const path of explicitFolders) {
    folders.add(path)
    for (const ancestor of workspaceAncestors(path)) folders.add(ancestor)
  }
  for (const file of files) {
    for (const ancestor of workspaceAncestors(file.path)) folders.add(ancestor)
  }
  const filePaths = new Set(files.map(({ path }) => path))
  for (const folder of folders) {
    if (filePaths.has(folder)) {
      throw new Error(`Workspace path is both a file and folder: ${folder}`)
    }
  }

  const sortedFiles = [...files].sort(compareFiles)
  const sortedFolders = [...folders].sort(comparePaths)
  const entryFile = optionalFilePath(seed.entryFile, sortedFiles, "entry file")
  const activeFile = optionalFilePath(
    seed.activeFile,
    sortedFiles,
    "active file"
  )
  const openFiles = workspaceFilePaths(
    seed.openFiles ?? [],
    sortedFiles,
    "open file"
  )
  if (activeFile !== undefined && !openFiles.includes(activeFile)) {
    throw new Error(`Active file must be open: ${activeFile}`)
  }
  const dirtyFiles = workspaceFilePaths(
    seed.dirtyFiles ?? [],
    sortedFiles,
    "dirty file"
  )
  const expandedFolders = workspaceFolderPaths(
    seed.expandedFolders ?? [],
    sortedFolders,
    "expanded folder"
  )

  return freezeWorkspace({
    files: sortedFiles,
    folders: sortedFolders,
    ...(entryFile === undefined
      ? {}
      : {
          entryFile,
          entryModule: workspaceModuleName(entryFile),
        }),
    ...(activeFile === undefined ? {} : { activeFile }),
    openFiles,
    dirtyFiles,
    expandedFolders,
    explorer: {
      visible: seed.explorer?.visible ?? false,
      width: clampExplorerWidth(seed.explorer?.width ?? defaultExplorerWidth),
    },
  })
}

export function workspacePath(path: string): WorkspacePath {
  return normalizeWorkspacePath(path)
}

/**
 * Normalizes a browser source path to the identity accepted by the project
 * compiler. This mirrors `source_identity` in `seseragi-wasm`:
 * source paths end in `.ssrg`, while their module path is relative, slash
 * separated, NFC-normalized, and has no empty, `.` or `..` segment.
 */
export function workspaceSourceIdentity(path: string): WorkspaceSourceIdentity {
  if (path.includes("\0")) {
    throw new Error("Workspace source path must not contain NUL")
  }
  if (!path.endsWith(".ssrg")) {
    throw new Error("Workspace source path must end in .ssrg")
  }
  const modulePath = path.slice(0, -5)
  if (modulePath === "") {
    throw new Error(
      "Workspace source path must include a module name before .ssrg"
    )
  }
  const module = normalizeWorkspaceModulePath(modulePath)
  return Object.freeze({
    path: `${module}.ssrg` as WorkspacePath,
    module,
  })
}

export function workspaceSourcePath(path: string): WorkspacePath {
  return workspaceSourceIdentity(path).path
}

function normalizeWorkspacePath(path: string): WorkspacePath {
  if (path === "") throw new Error("Workspace path must not be empty")
  if (path.startsWith("/")) {
    throw new Error(`Workspace path must be relative: ${path}`)
  }
  if (path.includes("\\")) {
    throw new Error(`Workspace path must use forward slashes: ${path}`)
  }
  if (path.includes("//")) {
    throw new Error(`Workspace path has duplicate separators: ${path}`)
  }
  if (path.includes("\0")) {
    throw new Error("Workspace path must not contain NUL")
  }
  const normalized = path.normalize("NFC")
  for (const segment of normalized.split("/")) {
    if (segment === "") {
      throw new Error(`Workspace path has an empty segment: ${path}`)
    }
    if (segment === "." || segment === "..") {
      throw new Error(`Workspace path must not contain . or ..: ${path}`)
    }
  }
  return normalized as WorkspacePath
}

function normalizeWorkspaceModulePath(path: string): string {
  if (path.endsWith(".ssrg")) {
    throw new Error("Workspace module path must omit .ssrg")
  }
  return normalizeWorkspacePath(path)
}

export function activeWorkspaceSource(state: WorkspaceState): string {
  if (state.activeFile === undefined) return ""
  return requireWorkspaceFile(state, state.activeFile).source
}

export function updateActiveWorkspaceSource(
  state: WorkspaceState,
  source: string
): WorkspaceState {
  if (state.activeFile === undefined) {
    throw new Error("Workspace has no active file")
  }
  return updateWorkspaceFileSource(state, state.activeFile, source)
}

export function updateWorkspaceFileSource(
  state: WorkspaceState,
  path: string,
  source: string
): WorkspaceState {
  const normalized = workspaceSourcePath(path)
  const file = requireWorkspaceFile(state, normalized)
  if (file.source === source && state.dirtyFiles.includes(normalized)) {
    return state
  }
  return freezeWorkspace({
    ...state,
    files: state.files.map((candidate) =>
      candidate.path === normalized ? { ...candidate, source } : candidate
    ),
    dirtyFiles: appendUnique(state.dirtyFiles, normalized),
  })
}

export function markWorkspaceFileClean(
  state: WorkspaceState,
  path: string
): WorkspaceState {
  const normalized = workspaceSourcePath(path)
  requireWorkspaceFile(state, normalized)
  if (!state.dirtyFiles.includes(normalized)) return state
  return freezeWorkspace({
    ...state,
    dirtyFiles: state.dirtyFiles.filter(
      (candidate) => candidate !== normalized
    ),
  })
}

export function createWorkspaceFile(
  state: WorkspaceState,
  path: string,
  source = ""
): WorkspaceState {
  const normalized = workspaceSourcePath(path)
  assertAvailablePath(state, normalized)
  assertParentFolder(state, normalized)
  return freezeWorkspace({
    ...state,
    files: [...state.files, { path: normalized, source }].sort(compareFiles),
    activeFile: normalized,
    openFiles: appendUnique(state.openFiles, normalized),
    dirtyFiles: appendUnique(state.dirtyFiles, normalized),
  })
}

export function createWorkspaceFolder(
  state: WorkspaceState,
  path: string
): WorkspaceState {
  const normalized = workspacePath(path)
  assertAvailablePath(state, normalized)
  assertParentFolder(state, normalized)
  return freezeWorkspace({
    ...state,
    folders: [...state.folders, normalized].sort(comparePaths),
  })
}

export function renameWorkspacePath(
  state: WorkspaceState,
  from: string,
  to: string
): WorkspaceState {
  const sourcePath = workspacePath(from)
  const sourceIsFile = hasWorkspaceFile(state, sourcePath)
  const sourceIsFolder = state.folders.includes(sourcePath)
  if (!sourceIsFile && !sourceIsFolder) {
    throw new Error(`Workspace path does not exist: ${sourcePath}`)
  }
  const targetPath = sourceIsFile ? workspaceSourcePath(to) : workspacePath(to)
  if (sourcePath === targetPath) return state
  if (sourceIsFolder && isDescendantPath(targetPath, sourcePath)) {
    throw new Error(`Workspace folder cannot move inside itself: ${targetPath}`)
  }
  assertParentFolder(state, targetPath)

  const move = (path: WorkspacePath): WorkspacePath =>
    remapWorkspacePath(path, sourcePath, targetPath)
  const movedFiles = state.files.map((file) =>
    sourceIsFile
      ? file.path === sourcePath
        ? { ...file, path: targetPath }
        : file
      : isSameOrDescendantPath(file.path, sourcePath)
        ? { ...file, path: move(file.path) }
        : file
  )
  const movedFolders = state.folders.map((folder) =>
    sourceIsFolder && isSameOrDescendantPath(folder, sourcePath)
      ? move(folder)
      : folder
  )
  validateMovedPaths(movedFiles, movedFolders)

  const remapReference = (path: WorkspacePath): WorkspacePath =>
    sourceIsFile
      ? path === sourcePath
        ? targetPath
        : path
      : isSameOrDescendantPath(path, sourcePath)
        ? move(path)
        : path
  const entryFile =
    state.entryFile === undefined ? undefined : remapReference(state.entryFile)

  return freezeWorkspace({
    ...state,
    files: movedFiles.sort(compareFiles),
    folders: movedFolders.sort(comparePaths),
    ...(entryFile === undefined
      ? { entryFile: undefined, entryModule: undefined }
      : {
          entryFile,
          entryModule: workspaceModuleName(entryFile),
        }),
    ...(state.activeFile === undefined
      ? { activeFile: undefined }
      : { activeFile: remapReference(state.activeFile) }),
    openFiles: state.openFiles.map(remapReference),
    dirtyFiles: state.dirtyFiles.map(remapReference),
    expandedFolders: state.expandedFolders.map(remapReference),
  })
}

export function deleteWorkspacePath(
  state: WorkspaceState,
  path: string
): WorkspaceState {
  const normalized = workspacePath(path)
  const isFile = hasWorkspaceFile(state, normalized)
  const isFolder = state.folders.includes(normalized)
  if (!isFile && !isFolder) {
    throw new Error(`Workspace path does not exist: ${normalized}`)
  }
  const removed = (candidate: WorkspacePath): boolean =>
    isFile
      ? candidate === normalized
      : isSameOrDescendantPath(candidate, normalized)
  const files = state.files.filter(({ path: filePath }) => !removed(filePath))
  const folders = state.folders.filter((folder) => !removed(folder))
  let openFiles = state.openFiles.filter((openFile) => !removed(openFile))
  let activeFile = state.activeFile
  if (activeFile !== undefined && removed(activeFile)) {
    activeFile = nextActiveFile(state, removed, files)
    if (activeFile !== undefined && !openFiles.includes(activeFile)) {
      openFiles = [...openFiles, activeFile]
    }
  }
  const entryFile =
    state.entryFile !== undefined && !removed(state.entryFile)
      ? state.entryFile
      : undefined

  return freezeWorkspace({
    ...state,
    files,
    folders,
    ...(entryFile === undefined
      ? { entryFile: undefined, entryModule: undefined }
      : { entryFile, entryModule: workspaceModuleName(entryFile) }),
    ...(activeFile === undefined ? { activeFile: undefined } : { activeFile }),
    openFiles,
    dirtyFiles: state.dirtyFiles.filter((dirtyFile) => !removed(dirtyFile)),
    expandedFolders: state.expandedFolders.filter(
      (expandedFolder) => !removed(expandedFolder)
    ),
  })
}

export function activateWorkspaceFile(
  state: WorkspaceState,
  path: string
): WorkspaceState {
  const normalized = workspaceSourcePath(path)
  requireWorkspaceFile(state, normalized)
  if (state.activeFile === normalized && state.openFiles.includes(normalized)) {
    return state
  }
  return freezeWorkspace({
    ...state,
    activeFile: normalized,
    openFiles: appendUnique(state.openFiles, normalized),
  })
}

export function closeWorkspaceFile(
  state: WorkspaceState,
  path: string
): WorkspaceState {
  const normalized = workspaceSourcePath(path)
  requireWorkspaceFile(state, normalized)
  const index = state.openFiles.indexOf(normalized)
  if (index < 0) return state
  const openFiles = state.openFiles.filter(
    (openFile) => openFile !== normalized
  )
  const activeFile =
    state.activeFile === normalized
      ? (openFiles[index] ?? openFiles[index - 1])
      : state.activeFile
  return freezeWorkspace({
    ...state,
    ...(activeFile === undefined ? { activeFile: undefined } : { activeFile }),
    openFiles,
  })
}

export function setWorkspaceEntryFile(
  state: WorkspaceState,
  path: string | undefined
): WorkspaceState {
  if (path === undefined) {
    return freezeWorkspace({
      ...state,
      entryFile: undefined,
      entryModule: undefined,
    })
  }
  const normalized = workspaceSourcePath(path)
  requireWorkspaceFile(state, normalized)
  return freezeWorkspace({
    ...state,
    entryFile: normalized,
    entryModule: workspaceModuleName(normalized),
  })
}

export function setWorkspaceFolderExpanded(
  state: WorkspaceState,
  path: string,
  expanded: boolean
): WorkspaceState {
  const normalized = workspacePath(path)
  if (!state.folders.includes(normalized)) {
    throw new Error(`Workspace folder does not exist: ${normalized}`)
  }
  const expandedFolders = expanded
    ? appendUnique(state.expandedFolders, normalized)
    : state.expandedFolders.filter((folder) => folder !== normalized)
  return freezeWorkspace({ ...state, expandedFolders })
}

export function setWorkspaceExplorer(
  state: WorkspaceState,
  update: Partial<WorkspaceExplorerState>
): WorkspaceState {
  return freezeWorkspace({
    ...state,
    explorer: {
      visible: update.visible ?? state.explorer.visible,
      width: clampExplorerWidth(update.width ?? state.explorer.width),
    },
  })
}

export function workspaceModuleName(path: WorkspacePath): string {
  return workspaceSourceIdentity(path).module
}

function freezeWorkspace(state: WorkspaceState): WorkspaceState {
  const files = state.files.map((file) => Object.freeze({ ...file }))
  return Object.freeze({
    ...state,
    files: Object.freeze(files),
    folders: Object.freeze([...state.folders]),
    openFiles: Object.freeze([...state.openFiles]),
    dirtyFiles: Object.freeze([...state.dirtyFiles]),
    expandedFolders: Object.freeze([...state.expandedFolders]),
    explorer: Object.freeze({ ...state.explorer }),
  })
}

function optionalFilePath(
  path: string | undefined,
  files: readonly WorkspaceFile[],
  label: string
): WorkspacePath | undefined {
  if (path === undefined) return undefined
  const normalized = workspaceSourcePath(path)
  if (!files.some((file) => file.path === normalized)) {
    throw new Error(`Workspace ${label} does not exist: ${normalized}`)
  }
  return normalized
}

function workspaceFilePaths(
  paths: readonly string[],
  files: readonly WorkspaceFile[],
  label: string
): readonly WorkspacePath[] {
  const normalized = paths.map(workspaceSourcePath)
  assertUniquePaths(label, normalized)
  for (const path of normalized) {
    if (!files.some((file) => file.path === path)) {
      throw new Error(`Workspace ${label} does not exist: ${path}`)
    }
  }
  return normalized
}

function workspaceFolderPaths(
  paths: readonly string[],
  folders: readonly WorkspacePath[],
  label: string
): readonly WorkspacePath[] {
  const normalized = paths.map(workspacePath)
  assertUniquePaths(label, normalized)
  for (const path of normalized) {
    if (!folders.includes(path)) {
      throw new Error(`Workspace ${label} does not exist: ${path}`)
    }
  }
  return normalized
}

function workspaceAncestors(path: WorkspacePath): readonly WorkspacePath[] {
  const segments = path.split("/")
  const ancestors: WorkspacePath[] = []
  for (let index = 1; index < segments.length; index += 1) {
    ancestors.push(segments.slice(0, index).join("/") as WorkspacePath)
  }
  return ancestors
}

function workspaceParent(path: WorkspacePath): WorkspacePath | undefined {
  const separator = path.lastIndexOf("/")
  return separator < 0 ? undefined : (path.slice(0, separator) as WorkspacePath)
}

function assertParentFolder(state: WorkspaceState, path: WorkspacePath): void {
  const parent = workspaceParent(path)
  if (parent !== undefined && !state.folders.includes(parent)) {
    throw new Error(`Workspace parent folder does not exist: ${parent}`)
  }
}

function assertAvailablePath(state: WorkspaceState, path: WorkspacePath): void {
  if (hasWorkspaceFile(state, path) || state.folders.includes(path)) {
    throw new Error(`Workspace path already exists: ${path}`)
  }
}

function requireWorkspaceFile(
  state: WorkspaceState,
  path: WorkspacePath
): WorkspaceFile {
  const file = state.files.find((candidate) => candidate.path === path)
  if (file === undefined)
    throw new Error(`Workspace file does not exist: ${path}`)
  return file
}

function hasWorkspaceFile(state: WorkspaceState, path: WorkspacePath): boolean {
  return state.files.some((file) => file.path === path)
}

function validateMovedPaths(
  files: readonly WorkspaceFile[],
  folders: readonly WorkspacePath[]
): void {
  for (const { path } of files) workspaceSourcePath(path)
  for (const path of folders) workspacePath(path)
  assertUniquePaths(
    "workspace file",
    files.map(({ path }) => path)
  )
  assertUniquePaths("workspace folder", folders)
  const filePaths = new Set(files.map(({ path }) => path))
  for (const folder of folders) {
    if (filePaths.has(folder)) {
      throw new Error(`Workspace path is both a file and folder: ${folder}`)
    }
  }
}

function remapWorkspacePath(
  path: WorkspacePath,
  from: WorkspacePath,
  to: WorkspacePath
): WorkspacePath {
  return (
    path === from ? to : `${to}${path.slice(from.length)}`
  ) as WorkspacePath
}

function isSameOrDescendantPath(
  path: WorkspacePath,
  parent: WorkspacePath
): boolean {
  return path === parent || isDescendantPath(path, parent)
}

function isDescendantPath(path: WorkspacePath, parent: WorkspacePath): boolean {
  return path.startsWith(`${parent}/`)
}

function nextActiveFile(
  state: WorkspaceState,
  removed: (path: WorkspacePath) => boolean,
  remainingFiles: readonly WorkspaceFile[]
): WorkspacePath | undefined {
  const activeIndex =
    state.activeFile === undefined
      ? -1
      : state.openFiles.indexOf(state.activeFile)
  for (
    let index = activeIndex + 1;
    index < state.openFiles.length;
    index += 1
  ) {
    const candidate = state.openFiles[index]
    if (candidate !== undefined && !removed(candidate)) return candidate
  }
  for (let index = activeIndex - 1; index >= 0; index -= 1) {
    const candidate = state.openFiles[index]
    if (candidate !== undefined && !removed(candidate)) return candidate
  }
  return remainingFiles[0]?.path
}

function appendUnique(
  paths: readonly WorkspacePath[],
  path: WorkspacePath
): readonly WorkspacePath[] {
  return paths.includes(path) ? paths : [...paths, path]
}

function assertUniquePaths(
  label: string,
  paths: readonly WorkspacePath[]
): void {
  const seen = new Set<WorkspacePath>()
  for (const path of paths) {
    if (seen.has(path)) throw new Error(`Duplicate ${label} path: ${path}`)
    seen.add(path)
  }
}

function comparePaths(left: WorkspacePath, right: WorkspacePath): number {
  return left.localeCompare(right)
}

function compareFiles(left: WorkspaceFile, right: WorkspaceFile): number {
  return comparePaths(left.path, right.path)
}

function clampExplorerWidth(width: number): number {
  if (!Number.isFinite(width)) return defaultExplorerWidth
  return Math.min(maximumExplorerWidth, Math.max(minimumExplorerWidth, width))
}
