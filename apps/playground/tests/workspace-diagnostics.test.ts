import { describe, expect, test } from "bun:test"
import type { Diagnostic, ProjectRequest } from "../src/compiler/types"
import { collectWorkspaceDiagnostics } from "../src/diagnostics/workspace-diagnostics"

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
})
