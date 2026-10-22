import { expect, test } from "@playwright/test"

test("evaluates program modules in an opaque preview realm", async ({
  page,
}) => {
  page.on("pageerror", (error) => console.log("Browser error:", error.message))
  page.on("console", (message) => {
    if (message.type() === "error")
      console.log("Browser console:", message.text())
  })
  await page.goto("/")
  await expect(page.locator("#run-button")).toBeEnabled()
  const boundary = await page.evaluate(async () => {
    const load = (path: string) => import(/* @vite-ignore */ path)
    const compiler = await load("/src/compiler/wasm-driver.ts")
    const { startIsolatedProject } = await load(
      "/src/runtime/isolated-execution.ts"
    )
    const compiled = await compiler.compileProject({
      schema: 1,
      manifest:
        '[package]\nname="fixture/isolation"\nversion="1.0.0"\nlanguage="^0.1.0"\n[run]\nentry="main"\n',
      files: [
        {
          path: "main.ssrg",
          source: "pub effect fn main -> Unit = succeed ()\n",
        },
      ],
    })
    if (compiled.status !== "success") throw new Error(JSON.stringify(compiled))
    const frame = document.createElement("iframe")
    frame.id = "boundary-frame"
    frame.name = "boundary-frame"
    document.body.append(frame)
    const execution = await startIsolatedProject(
      frame,
      {
        modules: compiled.modules.map(
          ({
            path,
            generated,
          }: {
            path: string
            generated: { typescript: string }
          }) => ({
            path,
            typescript: `globalThis.__isolatedMarker = "child"; if (typeof document !== "undefined") throw new Error("expected a Worker");\n${generated.typescript}`,
          })
        ),
        entryPath: compiled.entry.path,
        entry: compiled.entry.contract,
        input: "",
        initialHref: new URL("/runtime-preview.html", location.href).href,
      },
      { signal: new AbortController().signal }
    )
    const completion = await execution.completion
    return {
      completion,
      parentMarker: (globalThis as unknown as { __isolatedMarker?: string })
        .__isolatedMarker,
      documentInaccessible: frame.contentDocument === null,
      sandbox: frame.getAttribute("sandbox"),
    }
  })
  expect(boundary.completion.kind).toBe("completed")
  expect(boundary.parentMarker).toBeUndefined()
  expect(boundary.documentInaccessible).toBe(true)
  expect(boundary.sandbox).not.toContain("allow-same-origin")
  expect(boundary.sandbox).not.toContain("allow-popups-to-escape-sandbox")
  const child = page
    .frames()
    .find((frame) => frame.name() === "boundary-frame")!
  expect(
    await child.evaluate(
      () =>
        (globalThis as unknown as { __isolatedMarker: string }).__isolatedMarker
    )
  ).toBeUndefined()
})

test("preserves only the namespaced Storage capability across isolated runs", async ({
  page,
}) => {
  page.on("pageerror", (error) => console.log("Browser error:", error.message))
  page.on("console", (message) => {
    if (message.type() === "error")
      console.log("Browser console:", message.text())
  })
  await page.goto("/")
  await page.evaluate(() =>
    localStorage.setItem("isolation-test-host-control", "keep")
  )
  const source = `import * as storage from "std/web/storage"

type AppError deriving Show =
  | StorageFailure storage.StorageError

pub effect fn main -> Unit
with storage: storage.Storage
fails AppError = do {
  _ <- storage.clear storage.Local |> mapError StorageFailure
  _ <- storage.set storage.Local "profile" "seseragi" |> mapError StorageFailure
  succeed ()
}
`
  await page
    .getByRole("textbox", { name: "Seseragi source editor" })
    .fill(source)
  await page.locator("#run-button").click()
  await expect(page.locator("#status-text")).toHaveText("Completed")
  expect(
    await page.evaluate(() => ({
      application: localStorage.getItem(
        Object.keys(localStorage).find(
          (key) =>
            key.startsWith("seseragi:playground:application:v2:") &&
            key.endsWith(":profile")
        ) ?? "missing-scoped-key"
      ),
      host: localStorage.getItem("isolation-test-host-control"),
    }))
  ).toEqual({ application: "seseragi", host: "keep" })
  await page.locator("#run-button").click()
  await expect(page.locator("#status-text")).toHaveText("Completed")
  expect(
    await page.evaluate(() =>
      localStorage.getItem("isolation-test-host-control")
    )
  ).toBe("keep")
})

test("keeps Navigation history private while preserving push and replace", async ({
  page,
}) => {
  await page.goto("/")
  const source = `import * as navigation from "std/web/navigation"

type AppError deriving Show =
  | NavigationFailure navigation.NavigationError
  | ConsoleFailure ConsoleError

pub effect fn main -> Unit
with navigation: navigation.Navigation, Console
fails AppError = do {
  initial <- navigation.current () |> mapError NavigationFailure
  match navigation.resolveUrl "/preview-step?view=1#state" (navigation.locationUrl initial) {
    Left _ -> succeed ()
    Right target -> do {
      pushed <- navigation.push target |> mapError NavigationFailure
      _ <- println (navigation.renderUrl (navigation.locationUrl pushed)) |> mapError ConsoleFailure
      _ <- navigation.replace target |> mapError NavigationFailure
      next <- navigation.current () |> mapError NavigationFailure
      println (navigation.renderUrl (navigation.locationUrl next)) |> mapError ConsoleFailure
    }
  }
}
`
  await page
    .getByRole("textbox", { name: "Seseragi source editor" })
    .fill(source)
  const parentUrl = page.url()
  const origin = new URL(parentUrl).origin
  await page.locator("#run-button").click()
  await expect(page.locator("#status-text")).toHaveText("Completed")
  await expect(page.locator("#output")).toHaveText(
    `${origin}/preview-step?view=1#state\n${origin}/preview-step?view=1#state`
  )
  expect(page.url()).toBe(parentUrl)
})

test("settles execution when an ordinary same-frame link replaces the preview", async ({
  page,
}) => {
  await page.goto("/")
  await page.locator("#sample-browser-button").click()
  await page.locator('[data-sample-id="form-todo"]').click()
  await expect(page.locator("#status-text")).toHaveText("Analysis ready")
  await page.locator("#run-button").click()
  await expect(page.locator("#status-text")).toHaveText("Interactive")
  const preview = page.locator("#html-preview").contentFrame()
  const frame = page
    .frames()
    .find((candidate) => candidate.parentFrame() !== null)!
  await frame.evaluate(() => {
    const link = document.createElement("a")
    link.href = "/runtime-preview.html"
    link.textContent = "Next preview document"
    document.body.append(link)
  })
  await preview.getByRole("link", { name: "Next preview document" }).click()
  await expect(page.locator("#status-text")).toHaveText("Preview closed")
  await expect(page.locator("#run-button")).toBeEnabled()
  await expect(page.locator("#html-preview")).not.toHaveAttribute("srcdoc")
  await page.locator("#run-button").click()
  await expect(page.locator("#status-text")).toHaveText("Interactive")
})
