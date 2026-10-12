import { expect, test } from "@playwright/test"

test("explicit policy artifact produces a separate checked API with visible evidence", async ({
  page,
}) => {
  await page.goto("/")
  await expect(page.locator("#run-button")).toBeEnabled()
  const files = [
    {
      path: "seseragi.toml",
      source:
        '[package]\nname="fixture/semantic-policy"\nversion="1.0.0"\nlanguage="^0.1.0"\n[run]\nentry="main"\ntarget="web"\n[foreign.typescript]\nresolver="node"\nmanifest="host/package.json"\nlockfile="host/bun.lock"\nbindings="bindings.toml"\n',
    },
    {
      path: "bindings.toml",
      source:
        'schema=1\n[entries.api]\ndeclaration="host/api.d.ts"\nspecifier="./data.js"\noutput="api"\nevaluation="pure"\nprojection_policy="host/api.policy.json"\n',
    },
    {
      path: "host/package.json",
      source: '{"name":"fixture/semantic-policy","version":"1.0.0"}',
    },
    {
      path: "host/bun.lock",
      source:
        '{"lockfileVersion":1,"workspaces":{"":{"name":"fixture/semantic-policy"}},"packages":{}}',
    },
    {
      path: "host/api.d.ts",
      source:
        "export type User = { readonly name: string; readonly active: boolean };",
    },
    { path: "host/api.policy.json", source: "{}" },
    {
      path: "main.ssrg",
      source:
        'import * as json from "std/json"\nimport {decodeUser} from "gen/api/semantic"\nfn input text: String -> json.Json = match json.parse text { Left _ -> json.JsonNull; Right value -> value }\nfn decoded text: String -> String = match decodeUser (input text) { Left _ -> "rejected"; Right user -> user.name }\npub effect fn main -> Unit with Console fails ConsoleError = do {\n  println $ decoded "{\\"name\\":\\"Ada\\",\\"active\\":true}"\n  println $ decoded "{\\"name\\":3,\\"active\\":true}"\n}\n',
    },
  ]
  // Obtain fresh declaration identity from the actual compiler. No opaque/raw
  // object is approved automatically; the exact decoder mapping is explicit.
  const source = await page.evaluate(async (files) => {
    const load = (path: string) => import(/* @vite-ignore */ path)
    const compiler = await load("/src/compiler/wasm-driver.ts")
    const result = await compiler.convertWorkspaceBindings({
      schema: 1,
      revision: "policy-seed",
      files,
    })
    const graph = result.semantic[0].graph
    return JSON.stringify({
      schema: 1,
      kind: "seseragi-semantic-projection-policy",
      producer: { name: "explicit-browser-policy", version: "1" },
      graphDigest: result.compatibility[0].report.graphDigest,
      artifactIdentity: graph.identity,
      policies: [
        {
          selector: { kind: "symbol", symbol: "User" },
          functionName: "decodeUser",
          mapping: { kind: "json-snapshot" },
        },
      ],
    })
  }, files)
  files.find((file) => file.path === "host/api.policy.json")!.source = source
  const seed = {
    schema: 1,
    sampleId: "playground-blank",
    sampleHash: "workspace:blank-v1",
    stdin: "",
    workspace: {
      files,
      folders: ["host"],
      dirtyFiles: [],
      entryFile: "main.ssrg",
      activeFile: "main.ssrg",
      openFiles: ["main.ssrg"],
      expandedFolders: ["host"],
      explorer: { visible: true, width: 280 },
    },
  }
  await page.evaluate(
    (seed) =>
      localStorage.setItem(
        "seseragi.playground.workspace.schema-1",
        JSON.stringify(seed)
      ),
    seed
  )
  const errors: string[] = []
  page.on("pageerror", (error) => errors.push(error.message))
  await page.reload()
  await page
    .getByRole("button", { name: "Convert bindings", exact: true })
    .click()
  await expect(page.locator("#status-text")).not.toHaveText(
    "Converting bindings…"
  )
  await page.getByRole("button", { name: "Interop", exact: true }).click()
  await page.getByRole("tab", { name: "Compatibility", exact: true }).click()
  await expect(
    page.getByRole("heading", { name: "Checked semantic API · api" })
  ).toBeVisible()
  await expect(page.locator("#interop-inspector")).toContainText(
    "explicit-browser-policy"
  )
  await page
    .getByRole("button", { name: "Close Interop Inspector", exact: true })
    .click()
  await page.locator("#run-button").click()
  await expect(page.locator("#output")).toContainText("Ada")
  await expect(page.locator("#output")).toContainText("rejected")
  expect(errors).toEqual([])
})
