import { describe, expect, test } from "bun:test"
import {
  activateWorkspaceFile,
  createWorkspace,
  setWorkspaceEntryFile,
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
      entry: "main.ssrg",
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

    expect(workspaceProjectRequest(withoutEntry).entry).toBe(
      "feature/counter.ssrg"
    )
    expect(() => runnableWorkspaceProjectRequest(withoutEntry)).toThrow(
      "Select an entry file in Explorer before Run"
    )
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
})
