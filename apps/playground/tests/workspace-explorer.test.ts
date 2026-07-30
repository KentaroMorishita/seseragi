import { describe, expect, test } from "bun:test"
import {
  readExplorerWidth,
  workspaceDeletePrompt,
  workspaceTreeRows,
} from "../src/workspace/explorer"
import { createWorkspace } from "../src/workspace/model"

describe("Playground workspace Explorer", () => {
  test("renders folders before files with stable tree levels and active state", () => {
    const state = createWorkspace({
      files: [
        { path: "main.ssrg", source: "" },
        { path: "feature/zeta.ssrg", source: "" },
        { path: "feature/alpha.ssrg", source: "" },
        { path: "ui/view.ssrg", source: "" },
      ],
      activeFile: "feature/alpha.ssrg",
      openFiles: ["feature/alpha.ssrg"],
      expandedFolders: ["feature"],
    })

    expect(
      workspaceTreeRows(state).map(
        ({ path, kind, level, expanded, active, entry }) => ({
          path,
          kind,
          level,
          expanded,
          active,
          entry,
        })
      )
    ).toEqual([
      {
        path: "feature",
        kind: "folder",
        level: 1,
        expanded: true,
        active: false,
        entry: false,
      },
      {
        path: "feature/alpha.ssrg",
        kind: "file",
        level: 2,
        expanded: undefined,
        active: true,
        entry: false,
      },
      {
        path: "feature/zeta.ssrg",
        kind: "file",
        level: 2,
        expanded: undefined,
        active: false,
        entry: false,
      },
      {
        path: "ui",
        kind: "folder",
        level: 1,
        expanded: false,
        active: false,
        entry: false,
      },
      {
        path: "main.ssrg",
        kind: "file",
        level: 1,
        expanded: undefined,
        active: false,
        entry: false,
      },
    ])
  })

  test("marks the project entry independently from the active file", () => {
    const state = createWorkspace({
      files: [
        { path: "main.ssrg", source: "" },
        { path: "feature.ssrg", source: "" },
      ],
      entryFile: "main.ssrg",
      activeFile: "feature.ssrg",
      openFiles: ["feature.ssrg"],
    })

    expect(
      workspaceTreeRows(state)
        .filter(({ kind }) => kind === "file")
        .map(({ path, active, entry }) => ({ path, active, entry }))
    ).toEqual([
      { path: "feature.ssrg", active: true, entry: false },
      { path: "main.ssrg", active: false, entry: true },
    ])
  })

  test("hides descendants when their folder is collapsed", () => {
    const state = createWorkspace({
      files: [
        { path: "feature/counter.ssrg", source: "" },
        { path: "feature/internal/model.ssrg", source: "" },
      ],
      folders: ["feature", "feature/internal"],
    })

    expect(workspaceTreeRows(state).map(({ path }) => path)).toEqual([
      "feature",
    ])
  })

  test("restores and clamps the persisted Explorer width", () => {
    const storage = (value: string | null): Pick<Storage, "getItem"> => ({
      getItem: () => value,
    })

    expect(readExplorerWidth(storage("320"))).toBe(320)
    expect(readExplorerWidth(storage("80"))).toBe(180)
    expect(readExplorerWidth(storage("900"))).toBe(480)
    expect(readExplorerWidth(storage("not-a-number"))).toBe(240)
    expect(readExplorerWidth(storage(null))).toBe(240)
  })

  test("warns explicitly before deleting a non-empty folder subtree", () => {
    const state = createWorkspace({
      files: [{ path: "feature/internal/model.ssrg", source: "" }],
      folders: ["feature", "feature/internal"],
    })

    expect(workspaceDeletePrompt(state, "feature", "folder")).toBe(
      "Folder feature is not empty and contains 2 item(s). Delete the entire subtree?"
    )
  })

  test("connects the accessible tree, actions, resize and mobile drawer", async () => {
    const [html, main, styles, explorer] = await Promise.all([
      Bun.file(new URL("../index.html", import.meta.url)).text(),
      Bun.file(new URL("../src/main.ts", import.meta.url)).text(),
      Bun.file(new URL("../src/styles.css", import.meta.url)).text(),
      Bun.file(new URL("../src/workspace/explorer.ts", import.meta.url)).text(),
    ])

    expect(html).toContain('id="explorer-tree"')
    expect(html).toContain('role="tree"')
    expect(html).toContain('id="explorer-new-file"')
    expect(html).toContain('id="explorer-new-folder"')
    expect(html).toContain('id="explorer-collapse-all"')
    expect(html).toContain('id="explorer-resizer"')
    expect(main).toContain("connectWorkspaceExplorer(")
    expect(explorer).toContain('event.key === "ArrowDown"')
    expect(explorer).toContain('event.key === "F2"')
    expect(explorer).toContain('event.key === "Delete"')
    expect(explorer).toContain('action === "entry"')
    expect(styles).toContain(".explorer-entry-badge")
    expect(styles).toMatch(
      /\.explorer-panel \{[\s\S]*?position: absolute;[\s\S]*?box-shadow:/
    )
  })
})
