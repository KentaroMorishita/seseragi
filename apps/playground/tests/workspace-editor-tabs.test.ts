import { describe, expect, test } from "bun:test"
import { EditorSelection, EditorState } from "@codemirror/state"
import { createWorkspaceEditorSessions } from "../src/workspace/editor-session"
import {
  workspaceDirtyClosePrompt,
  workspaceTabItems,
} from "../src/workspace/editor-tabs"
import {
  activateWorkspaceFile,
  closeWorkspaceFile,
  createWorkspace,
  renameWorkspacePath,
  setWorkspaceExplorer,
} from "../src/workspace/model"

describe("Playground workspace editor tabs", () => {
  test("derives active and dirty tabs from the shared workspace state", () => {
    const state = createWorkspace({
      files: [
        { path: "main.ssrg", source: "main" },
        { path: "feature/counter.ssrg", source: "counter" },
      ],
      activeFile: "feature/counter.ssrg",
      openFiles: ["main.ssrg", "feature/counter.ssrg"],
      dirtyFiles: ["feature/counter.ssrg"],
    })

    expect(workspaceTabItems(state)).toEqual([
      { path: "main.ssrg", name: "main.ssrg", active: false, dirty: false },
      {
        path: "feature/counter.ssrg",
        name: "counter.ssrg",
        active: true,
        dirty: true,
      },
    ])
  })

  test("closes a dirty tab without discarding its workspace source", () => {
    const state = createWorkspace({
      files: [
        { path: "main.ssrg", source: "main" },
        { path: "counter.ssrg", source: "edited" },
      ],
      activeFile: "counter.ssrg",
      openFiles: ["main.ssrg", "counter.ssrg"],
      dirtyFiles: ["counter.ssrg"],
    })

    const closed = closeWorkspaceFile(state, "counter.ssrg")
    expect(closed.activeFile).toBe("main.ssrg")
    expect(closed.openFiles).toEqual(["main.ssrg"])
    expect(closed.dirtyFiles).toEqual(["counter.ssrg"])
    expect(
      closed.files.find(({ path }) => path === "counter.ssrg")?.source
    ).toBe("edited")
    expect(workspaceDirtyClosePrompt("counter.ssrg")).toContain(
      "edits will stay in the workspace"
    )
  })

  test("restores selection, editor history state and scroll with one view", async () => {
    const initial = createWorkspace({
      files: [
        { path: "main.ssrg", source: "main source" },
        { path: "counter.ssrg", source: "counter source" },
      ],
      activeFile: "main.ssrg",
      openFiles: ["main.ssrg", "counter.ssrg"],
    })
    const mainState = EditorState.create({
      doc: "main source",
      selection: EditorSelection.cursor(4),
    })
    const scrollDOM = { scrollLeft: 8, scrollTop: 120 }
    const view = {
      state: mainState,
      scrollDOM,
      setState(state: EditorState) {
        this.state = state
        scrollDOM.scrollLeft = 0
        scrollDOM.scrollTop = 0
      },
    }
    const sessions = createWorkspaceEditorSessions(view, (source) =>
      EditorState.create({ doc: source })
    )

    const counter = activateWorkspaceFile(initial, "counter.ssrg")
    expect(sessions.transition(initial, counter)).toBe(true)
    expect(view.state.doc.toString()).toBe("counter source")
    view.state = view.state.update({
      selection: EditorSelection.cursor(7),
    }).state
    scrollDOM.scrollLeft = 3
    scrollDOM.scrollTop = 64

    const main = activateWorkspaceFile(counter, "main.ssrg")
    expect(sessions.transition(counter, main)).toBe(true)
    await Promise.resolve()
    expect(view.state).toBe(mainState)
    expect(view.state.selection.main.head).toBe(4)
    expect(scrollDOM).toEqual({ scrollLeft: 8, scrollTop: 120 })
  })

  test("preserves the active editor session while Explorer toggles", () => {
    const initial = createWorkspace({
      files: [
        { path: "main.ssrg", source: "main source" },
        { path: "counter.ssrg", source: "counter source" },
      ],
      activeFile: "counter.ssrg",
      openFiles: ["main.ssrg", "counter.ssrg"],
      dirtyFiles: ["counter.ssrg"],
    })
    const editorState = EditorState.create({
      doc: "counter source",
      selection: EditorSelection.cursor(7),
    })
    const scrollDOM = { scrollLeft: 5, scrollTop: 96 }
    const view = {
      state: editorState,
      scrollDOM,
      setState(state: EditorState) {
        this.state = state
      },
    }
    const sessions = createWorkspaceEditorSessions(view, (source) =>
      EditorState.create({ doc: source })
    )

    const opened = setWorkspaceExplorer(initial, { visible: true })
    const closed = setWorkspaceExplorer(opened, { visible: false })

    expect(sessions.transition(initial, opened)).toBe(false)
    expect(sessions.transition(opened, closed)).toBe(false)
    expect(closed.activeFile).toBe("counter.ssrg")
    expect(closed.openFiles).toEqual(["main.ssrg", "counter.ssrg"])
    expect(closed.dirtyFiles).toEqual(["counter.ssrg"])
    expect(view.state).toBe(editorState)
    expect(view.state.selection.main.head).toBe(7)
    expect(scrollDOM).toEqual({ scrollLeft: 5, scrollTop: 96 })
  })

  test("remaps a remembered editor state through a folder rename", () => {
    const feature = createWorkspace({
      files: [
        { path: "main.ssrg", source: "main" },
        { path: "feature/counter.ssrg", source: "counter" },
      ],
      activeFile: "feature/counter.ssrg",
      openFiles: ["main.ssrg", "feature/counter.ssrg"],
    })
    const featureState = EditorState.create({
      doc: "counter",
      selection: EditorSelection.cursor(5),
    })
    const scrollDOM = { scrollLeft: 0, scrollTop: 0 }
    const view = {
      state: featureState,
      scrollDOM,
      setState(state: EditorState) {
        this.state = state
      },
    }
    const sessions = createWorkspaceEditorSessions(view, (source) =>
      EditorState.create({ doc: source })
    )
    const main = activateWorkspaceFile(feature, "main.ssrg")
    sessions.transition(feature, main)
    const renamed = renameWorkspacePath(main, "feature", "domain")

    expect(
      sessions.transition(main, renamed, { from: "feature", to: "domain" })
    ).toBe(false)
    const reopened = activateWorkspaceFile(renamed, "domain/counter.ssrg")
    expect(sessions.transition(renamed, reopened)).toBe(true)
    expect(view.state).toBe(featureState)
    expect(view.state.selection.main.head).toBe(5)
  })

  test("connects overflow tabs, dirty Explorer state and file identity", async () => {
    const [html, main, styles, explorer] = await Promise.all([
      Bun.file(new URL("../index.html", import.meta.url)).text(),
      Bun.file(new URL("../src/main.ts", import.meta.url)).text(),
      Bun.file(new URL("../src/styles.css", import.meta.url)).text(),
      Bun.file(new URL("../src/workspace/explorer.ts", import.meta.url)).text(),
    ])

    expect(html).toContain('id="workspace-tabs"')
    expect(html).toContain('role="tablist"')
    expect(main).toContain("createWorkspaceEditorSessions(")
    expect(main).toContain("workspaceState.activeFile")
    expect(styles).toMatch(/\.workspace-tabs \{[\s\S]*?overflow-x: auto;/)
    expect(explorer).toContain("state.dirtyFiles.includes(path)")
  })
})
