import { describe, expect, test } from "bun:test"
import {
  activateWorkspaceFile,
  closeWorkspaceFile,
  createWorkspace,
  setWorkspaceEntryFile,
  setWorkspaceExplorer,
} from "../src/workspace/model"
import {
  runnableWorkspaceProjectRequest,
  workspaceAnalysisRequest,
  workspaceAnalysisRevision,
  workspaceProjectRequest,
  workspaceProjectRevision,
} from "../src/workspace/project-request"

describe("Playground workspace compiler requests", () => {
  const state = createWorkspace({
    files: [
      { path: "feature/counter.ssrg", source: "pub let count = 41\n" },
      { path: "main.ssrg", source: "pub let answer = 42\n" },
    ],
    entryFile: "main.ssrg",
    activeFile: "feature/counter.ssrg",
    openFiles: ["main.ssrg", "feature/counter.ssrg"],
  })

  test("includes every file while keeping entry and active file distinct", () => {
    expect(workspaceProjectRequest(state)).toEqual({
      schema: 1,
      manifest:
        '[package]\nname = "playground/workspace"\nversion = "0.0.0"\nlanguage = "^0.1.0"\n\n[run]\nentry = "main"\n',
      files: [
        {
          path: "feature/counter.ssrg",
          source: "pub let count = 41\n",
        },
        { path: "main.ssrg", source: "pub let answer = 42\n" },
      ],
    })
    expect(workspaceAnalysisRequest(state)).toEqual({
      active: "feature/counter.ssrg",
      project: workspaceProjectRequest(state),
    })
  })

  test("uses the active file for recoverable analysis without an entry", () => {
    const withoutEntry = setWorkspaceEntryFile(state, undefined)

    expect(workspaceProjectRequest(withoutEntry).manifest).toContain(
      'entry = "feature/counter"'
    )
    expect(() => runnableWorkspaceProjectRequest(withoutEntry)).toThrow(
      "Select an entry file in Explorer before Run"
    )
  })

  test("passes the compiler the same canonical paths held by workspace state", () => {
    const normalized = createWorkspace({
      files: [{ path: "feature/cafe\u0301.ssrg", source: "pub let answer = 42\n" }],
      entryFile: "feature/cafe\u0301.ssrg",
      activeFile: "feature/cafe\u0301.ssrg",
      openFiles: ["feature/cafe\u0301.ssrg"],
    })

    expect(normalized.files.map(({ path }) => path)).toEqual([
      "feature/café.ssrg",
    ])
    expect(workspaceProjectRequest(normalized)).toEqual({
      schema: 1,
      manifest:
        '[package]\nname = "playground/workspace"\nversion = "0.0.0"\nlanguage = "^0.1.0"\n\n[run]\nentry = "feature/café"\n',
      files: [{ path: "feature/café.ssrg", source: "pub let answer = 42\n" }],
    })
    expect(workspaceAnalysisRequest(normalized).active).toBe("feature/café.ssrg")
  })

  test("revisions change for graph, entry and active-file changes", () => {
    const withoutEntry = setWorkspaceEntryFile(state, undefined)
    const mainActive = activateWorkspaceFile(state, "main.ssrg")

    expect(workspaceProjectRevision(withoutEntry)).not.toBe(
      workspaceProjectRevision(state)
    )
    expect(workspaceAnalysisRevision(withoutEntry)).not.toBe(
      workspaceAnalysisRevision(state)
    )
    expect(workspaceProjectRevision(mainActive)).toBe(
      workspaceProjectRevision(state)
    )
    expect(workspaceAnalysisRevision(mainActive)).not.toBe(
      workspaceAnalysisRevision(state)
    )
  })

  test("does not reanalyze for tabs or Explorer chrome-only changes", () => {
    const closedBackgroundTab = closeWorkspaceFile(state, "main.ssrg")
    const resizedExplorer = setWorkspaceExplorer(state, {
      visible: true,
      width: 360,
    })

    expect(workspaceProjectRevision(closedBackgroundTab)).toBe(
      workspaceProjectRevision(state)
    )
    expect(workspaceAnalysisRevision(closedBackgroundTab)).toBe(
      workspaceAnalysisRevision(state)
    )
    expect(workspaceProjectRevision(resizedExplorer)).toBe(
      workspaceProjectRevision(state)
    )
    expect(workspaceAnalysisRevision(resizedExplorer)).toBe(
      workspaceAnalysisRevision(state)
    )
  })
})
