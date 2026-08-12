import { describe, expect, test } from "bun:test"
import type { Diagnostic, ProjectRequest } from "../src/compiler/types"
import { collectWorkspaceDiagnostics } from "../src/diagnostics/workspace-diagnostics"
import { activateWorkspaceFile, createWorkspace } from "../src/workspace/model"
import { workspaceProjectRequest } from "../src/workspace/project-request"

const request: ProjectRequest = {
  schema: 1,
  entry: "main.ssrg",
  files: [
    { path: "feature/value.ssrg", source: "pub let value = missing\n" },
    { path: "main.ssrg", source: 'import { value } from "./feature/value"\n' },
  ],
}

const diagnostic: Diagnostic = {
  code: "SES-N0101",
  messageKey: "name.not-found",
  message: "Cannot find value `missing`",
  severity: "Error",
  primary: { start: 16, end: 23 },
  related: [],
  labels: [],
  notes: [],
  helps: [],
  fixes: [],
  expectedType: null,
  actualType: null,
}

describe("Playground workspace diagnostics", () => {
  test("keeps each diagnostic paired with its file source", () => {
    expect(
      collectWorkspaceDiagnostics(request, [
        {
          path: "feature/value.ssrg",
          diagnostics: { diagnostics: [diagnostic] },
        },
      ])
    ).toEqual([
      {
        path: "feature/value.ssrg",
        source: "pub let value = missing\n",
        diagnostic,
      },
    ])
  })

  test("keeps multiple files ordered and independently navigable", () => {
    const parseDiagnostic = {
      ...diagnostic,
      code: "SES-P0001",
      messageKey: "parser.expected-expression",
      message: "Expected an expression here",
      primary: { start: 20, end: 20 },
    }

    expect(
      collectWorkspaceDiagnostics(request, [
        {
          path: "main.ssrg",
          diagnostics: { diagnostics: [parseDiagnostic] },
        },
        {
          path: "feature/value.ssrg",
          diagnostics: { diagnostics: [diagnostic] },
        },
      ])
    ).toEqual([
      {
        path: "main.ssrg",
        source: 'import { value } from "./feature/value"\n',
        diagnostic: parseDiagnostic,
      },
      {
        path: "feature/value.ssrg",
        source: "pub let value = missing\n",
        diagnostic,
      },
    ])
  })

  test("turns graph problems into navigable diagnostics and removes duplicates", () => {
    const problem = {
      code: "SES-N0104",
      message: "Imported module does not exist",
      path: "main.ssrg",
      primary: { start: 23, end: 40 },
    }

    expect(
      collectWorkspaceDiagnostics(request, [], [problem, problem])
    ).toEqual([
      {
        path: "main.ssrg",
        source: 'import { value } from "./feature/value"\n',
        diagnostic: expect.objectContaining({
          code: "SES-N0104",
          message: "Imported module does not exist",
          primary: { start: 23, end: 40 },
        }),
      },
    ])
  })

  test("keeps provider context on the navigable source range", () => {
    const diagnostics = collectWorkspaceDiagnostics(
      request,
      [],
      [
        {
          code: "SES-K0201",
          label: "provider.missing",
          message: "Required provider is missing",
          path: "main.ssrg",
          primary: { start: 10, end: 18 },
          details: {
            service: "std/clock::Clock",
            target: "bun-process",
            backendFamily: "typescript",
            backendAbiMajor: 1,
            candidates: [],
          },
        },
      ]
    )

    expect(diagnostics).toEqual([
      {
        path: "main.ssrg",
        source: 'import { value } from "./feature/value"\n',
        diagnostic: expect.objectContaining({
          code: "SES-K0201",
          messageKey: "provider.missing",
          primary: { start: 10, end: 18 },
          notes: [
            "service: std/clock::Clock",
            "target: bun-process",
            "backend: typescript",
            "backend ABI major: 1",
          ],
        }),
      },
    ])
  })

  test("uses the canonical workspace tab path for compiler diagnostics", () => {
    const workspace = createWorkspace({
      files: [
        {
          path: "feature/cafe\u0301.ssrg",
          source: 'pub let broken: Int = "wrong"\n',
        },
      ],
      entryFile: "feature/cafe\u0301.ssrg",
      activeFile: "feature/cafe\u0301.ssrg",
      openFiles: ["feature/cafe\u0301.ssrg"],
    })
    const project = workspaceProjectRequest(workspace)
    const path = "feature/café.ssrg"
    const diagnostics = collectWorkspaceDiagnostics(project, [
      { path, diagnostics: { diagnostics: [diagnostic] } },
    ])

    expect(workspace.files.some((file) => file.path === path)).toBe(true)
    expect(activateWorkspaceFile(workspace, path).activeFile).toBe(path)
    expect(diagnostics).toEqual([
      {
        path,
        source: 'pub let broken: Int = "wrong"\n',
        diagnostic,
      },
    ])
  })
})
