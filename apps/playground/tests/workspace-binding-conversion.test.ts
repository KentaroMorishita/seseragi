import { expect, test } from "bun:test"
import {
  type BindingResponse,
  bindingDiagnostics,
  createBindingConversion,
  workspaceBindingRequest,
} from "../src/workspace/bindings"
import {
  createWorkspace,
  renameWorkspacePath,
  setWorkspaceExplorer,
  updateWorkspaceFileSource,
} from "../src/workspace/model"

const workspace = () =>
  createWorkspace({
    files: [
      { path: "input.d.ts", source: "export const café: any;" },
      { path: "settings", source: "schema=1" },
    ],
  })
const result = (revision: string): BindingResponse => ({
  schema: 1,
  revision,
  status: "success",
  generated: [],
  diagnostics: [],
})

test("conversion revision covers artifact content and paths but not Explorer chrome", () => {
  const state = workspace()
  const request = workspaceBindingRequest(state)
  expect(
    workspaceBindingRequest(setWorkspaceExplorer(state, { visible: true }))
      .revision
  ).toBe(request.revision)
  expect(
    workspaceBindingRequest(
      updateWorkspaceFileSource(state, "settings", "schema=2")
    ).revision
  ).not.toBe(request.revision)
  expect(
    workspaceBindingRequest(
      renameWorkspacePath(state, "input.d.ts", "renamed.d.ts")
    ).revision
  ).not.toBe(request.revision)
  expect(workspaceBindingRequest(state, "selected").revision).not.toBe(
    request.revision
  )
})

test("late success, failure and edit-restore cannot publish over the current conversion", async () => {
  let state = workspace()
  const pending: {
    resolve: (response: BindingResponse) => void
    reject: (error: Error) => void
  }[] = []
  const controller = createBindingConversion(
    () => new Promise((resolve, reject) => pending.push({ resolve, reject })),
    () => workspaceBindingRequest(state).revision
  )
  const request = workspaceBindingRequest(state)
  const first = controller.run(request)
  state = updateWorkspaceFileSource(state, "settings", "schema=2")
  controller.invalidate()
  const secondRequest = workspaceBindingRequest(state)
  const second = controller.run(secondRequest)
  pending[1]?.resolve(result(secondRequest.revision))
  expect(await second).toEqual(result(secondRequest.revision))
  pending[0]?.resolve(result(request.revision))
  expect(await first).toBeUndefined()
  const failing = controller.run(secondRequest)
  controller.invalidate()
  state = workspace()
  pending[2]?.reject(new Error("late error"))
  expect(await failing).toBeUndefined()
  const aba = controller.run(request)
  controller.invalidate()
  state = workspace()
  pending[3]?.resolve(result(request.revision))
  expect(await aba).toBeUndefined()
})

test("diagnostics retain the captured external source and UTF-8 span", () => {
  const request = workspaceBindingRequest(workspace())
  const diagnostics = bindingDiagnostics(request, {
    ...result(request.revision),
    diagnostics: [
      {
        code: "SES-F0100",
        severity: "Error",
        message: "unsupported",
        file: "input.d.ts",
        start: 20,
        end: 23,
      },
    ],
  })
  expect(diagnostics[0]?.source).toBe("export const café: any;")
  expect(diagnostics[0]?.path).toBe("input.d.ts")
  expect(diagnostics[0]?.diagnostic.primary).toEqual({ start: 20, end: 23 })
})
