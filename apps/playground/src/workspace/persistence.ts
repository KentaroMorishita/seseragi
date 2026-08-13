import {
  createWorkspace,
  type WorkspaceSeed,
  type WorkspaceState,
} from "./model"

export const workspacePersistenceSchema = 1
export const workspacePersistenceKey = "seseragi.playground.workspace.schema-1"

export type WorkspaceStorage = Pick<
  Storage,
  "getItem" | "setItem" | "removeItem"
>

export type PersistableSample = Readonly<{
  id: string
  workspaceHash: string
}>

export type WorkspaceRestore =
  | Readonly<{ status: "empty" }>
  | Readonly<{
      status: "restored"
      sampleId: string
      workspace: WorkspaceState
      stdin: string
    }>
  | Readonly<{ status: "recovered"; diagnostic: string }>

export type WorkspacePersistenceResult =
  | Readonly<{ status: "saved" }>
  | Readonly<{ status: "failure"; diagnostic: string }>

type JsonObject = Record<string, unknown>

export function restoreWorkspace(
  storage: WorkspaceStorage,
  samples: readonly PersistableSample[]
): WorkspaceRestore {
  let raw: string | null
  try {
    raw = storage.getItem(workspacePersistenceKey)
  } catch {
    return {
      status: "recovered",
      diagnostic:
        "Local workspaceを読み込めませんでした。初期sampleで開始します。",
    }
  }
  if (raw === null) return { status: "empty" }

  try {
    const persisted = expectObject(JSON.parse(raw), "persisted workspace")
    if (persisted.schema !== workspacePersistenceSchema) {
      throw new Error("workspace schema is incompatible")
    }
    const sampleId = expectString(persisted.sampleId, "sampleId")
    const sample = samples.find(({ id }) => id === sampleId)
    if (sample === undefined) throw new Error("sample is no longer available")
    if (
      expectString(persisted.sampleHash, "sampleHash") !== sample.workspaceHash
    ) {
      throw new Error("sample has changed")
    }
    return {
      status: "restored",
      sampleId,
      workspace: createWorkspace(
        parseWorkspaceSeed(persisted.workspace, "workspace")
      ),
      stdin: expectString(persisted.stdin, "stdin"),
    }
  } catch {
    removePersistedWorkspace(storage)
    return {
      status: "recovered",
      diagnostic:
        "保存されていたworkspaceは古いか破損していたため、初期sampleへ安全に戻しました。",
    }
  }
}

export function persistWorkspace(
  storage: WorkspaceStorage,
  sample: PersistableSample,
  workspace: WorkspaceState,
  stdin: string
): WorkspacePersistenceResult {
  try {
    storage.setItem(
      workspacePersistenceKey,
      JSON.stringify({
        schema: workspacePersistenceSchema,
        sampleId: sample.id,
        sampleHash: sample.workspaceHash,
        workspace: workspaceSeed(workspace),
        stdin,
      })
    )
    return { status: "saved" }
  } catch {
    return {
      status: "failure",
      diagnostic:
        "Local workspaceを保存できませんでした。ブラウザの保存容量を確認してください。このtabを閉じるまでは編集を続けられます。",
    }
  }
}

export function confirmDirtyWorkspaceSwitch(
  workspace: WorkspaceState,
  nextSampleTitle: string,
  confirm: (message: string) => boolean
): boolean {
  if (workspace.dirtyFiles.length === 0) return true
  const files = workspace.dirtyFiles.join(", ")
  return confirm(
    `workspaceに未保存の変更があります (${files})。` +
      `${nextSampleTitle}へ切り替えると、このworkspaceの変更は破棄されます。続けますか？`
  )
}

function workspaceSeed(workspace: WorkspaceState): WorkspaceSeed {
  return {
    files: workspace.files,
    folders: workspace.folders,
    ...(workspace.entryFile === undefined
      ? {}
      : { entryFile: workspace.entryFile }),
    ...(workspace.packageManifest === undefined
      ? {}
      : {
          packageManifest: workspace.packageManifest,
          packageEntryFile: workspace.packageEntryFile,
        }),
    ...(workspace.activeFile === undefined
      ? {}
      : { activeFile: workspace.activeFile }),
    openFiles: workspace.openFiles,
    dirtyFiles: workspace.dirtyFiles,
    expandedFolders: workspace.expandedFolders,
    explorer: workspace.explorer,
  }
}

function parseWorkspaceSeed(value: unknown, context: string): WorkspaceSeed {
  const workspace = expectObject(value, context)
  const filesValue = workspace.files
  if (!Array.isArray(filesValue)) throw new Error(`${context}.files is invalid`)
  const files = filesValue.map((value, index) => {
    const file = expectObject(value, `${context}.files[${index}]`)
    return {
      path: expectString(file.path, `${context}.files[${index}].path`),
      source: expectString(file.source, `${context}.files[${index}].source`),
    }
  })
  const explorer = expectObject(workspace.explorer, `${context}.explorer`)
  return {
    files,
    folders: expectStrings(workspace.folders, `${context}.folders`),
    ...optionalStringProperty(workspace, "entryFile", context),
    ...optionalStringProperty(workspace, "packageManifest", context),
    ...optionalStringProperty(workspace, "packageEntryFile", context),
    ...optionalStringProperty(workspace, "activeFile", context),
    openFiles: expectStrings(workspace.openFiles, `${context}.openFiles`),
    dirtyFiles: expectStrings(workspace.dirtyFiles, `${context}.dirtyFiles`),
    expandedFolders: expectStrings(
      workspace.expandedFolders,
      `${context}.expandedFolders`
    ),
    explorer: {
      visible: expectBoolean(explorer.visible, `${context}.explorer.visible`),
      width: expectNumber(explorer.width, `${context}.explorer.width`),
    },
  }
}

function optionalStringProperty(
  value: JsonObject,
  key: "entryFile" | "activeFile" | "packageManifest" | "packageEntryFile",
  context: string
): Partial<
  Pick<
    WorkspaceSeed,
    "entryFile" | "activeFile" | "packageManifest" | "packageEntryFile"
  >
> {
  const candidate = value[key]
  return candidate === undefined
    ? {}
    : { [key]: expectString(candidate, `${context}.${key}`) }
}

function expectObject(value: unknown, context: string): JsonObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error(`${context} must be an object`)
  }
  return value as JsonObject
}

function expectString(value: unknown, context: string): string {
  if (typeof value !== "string") throw new Error(`${context} must be a string`)
  return value
}

function expectStrings(value: unknown, context: string): string[] {
  if (!Array.isArray(value)) throw new Error(`${context} must be an array`)
  return value.map((item, index) => expectString(item, `${context}[${index}]`))
}

function expectBoolean(value: unknown, context: string): boolean {
  if (typeof value !== "boolean")
    throw new Error(`${context} must be a boolean`)
  return value
}

function expectNumber(value: unknown, context: string): number {
  if (typeof value !== "number" || !Number.isFinite(value)) {
    throw new Error(`${context} must be a finite number`)
  }
  return value
}

function removePersistedWorkspace(storage: WorkspaceStorage): void {
  try {
    storage.removeItem(workspacePersistenceKey)
  } catch {
    // Recovery still proceeds with the in-memory initial sample.
  }
}
