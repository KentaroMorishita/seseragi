import { cpSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs"
import { join, resolve, sep } from "node:path"
import {
  chromium,
  expect,
} from "../apps/playground/node_modules/@playwright/test"

const root = resolve(import.meta.dir, "..")
const output = resolve(
  process.env.SESERAGI_PRODUCTION_BROWSER_OUTPUT ??
    join(root, "target/production-browser")
)
const cli = resolve(
  process.env.SESERAGI_BIN ??
    join(process.env.CARGO_TARGET_DIR ?? join(root, "target"), "debug/seseragi")
)
function run(args: string[]) {
  const result = Bun.spawnSync([cli, ...args], {
    cwd: root,
    stdout: "pipe",
    stderr: "pipe",
  })
  if (result.exitCode !== 0) throw new Error(result.stderr.toString())
}
mkdirSync(output, { recursive: true })
const browser = await chromium.launch({ headless: true })
const report = []
try {
  for (const [id, sample] of [
    ["web-starter", "web-starter"],
    ["flow-app", "project-flow-app"],
  ]) {
    const input = join(output, ".inputs", id)
    rmSync(input, { recursive: true, force: true })
    cpSync(join(root, "examples/samples", sample), input, { recursive: true })
    run(["lock", "update", input])
    const observations = []
    for (const profile of ["development", "release"]) {
      const artifact = join(output, `${id}-${profile}`)
      run([
        "build",
        input,
        "--target",
        "web",
        "--profile",
        profile,
        "--out-dir",
        artifact,
      ])
      const manifest = JSON.parse(
        readFileSync(join(artifact, "artifact-manifest.json"), "utf8")
      )
      expect(manifest.sourceMap.policy).toBe(
        profile === "release" ? "omit" : "emit"
      )
      expect(
        manifest.files.some((file: { path: string }) =>
          file.path.endsWith(".map")
        )
      ).toBe(profile !== "release")
      const server = Bun.serve({
        hostname: "127.0.0.1",
        port: 0,
        async fetch(request) {
          const pathname = decodeURIComponent(new URL(request.url).pathname)
          const file = resolve(
            artifact,
            `.${pathname === "/" ? "/index.html" : pathname}`
          )
          if (!file.startsWith(`${artifact}${sep}`))
            return new Response("Forbidden", { status: 403 })
          const content = Bun.file(file)
          return (await content.exists())
            ? new Response(content)
            : new Response("Not found", { status: 404 })
        },
      })
      const context = await browser.newContext({
        viewport: { width: 1280, height: 900 },
      })
      const page = await context.newPage()
      const errors: string[] = []
      page.on("pageerror", (error) => errors.push(error.message))
      // Third-party photography is outside application semantics and must not make
      // first-party/offline verification flaky. Keep its dimensions visible.
      await page.route("https://images.unsplash.com/**", (route) =>
        route.fulfill({
          contentType: "image/svg+xml",
          body: '<svg xmlns="http://www.w3.org/2000/svg" width="960" height="480"><rect width="960" height="480" fill="#dbeafe"/></svg>',
        })
      )
      try {
        await page.goto(`http://127.0.0.1:${server.port}/`)
        if (id === "web-starter") {
          await expect(
            page.getByText("Count: 0", { exact: true })
          ).toBeVisible()
          await page
            .getByRole("button", { name: "Count one more", exact: true })
            .click({ clickCount: 2 })
          await expect(
            page.getByText("Count: 2", { exact: true })
          ).toBeVisible()
        } else {
          await page
            .getByRole("button", { name: "Use day studio", exact: true })
            .click()
          await expect(
            page.getByRole("button", { name: "Use night studio", exact: true })
          ).toBeVisible()
          const focus = page
            .locator("#focus-add")
            .locator("xpath=ancestor::section[1]")
          await page.locator("#focus-add").click()
          await expect(focus).toContainText("3")
          await page.locator("#focus-remove").click()
          await expect(focus).toContainText("2")
          await page
            .getByRole("button", { name: "Add a story card", exact: true })
            .click()
          await expect(page.getByRole("alert")).toBeVisible()
          await page
            .getByLabel("Story card title", { exact: true })
            .fill("Production parity story")
          await page
            .getByRole("button", { name: "Add a story card", exact: true })
            .click()
          const added = page
            .getByLabel("Editable story card", { exact: true })
            .last()
          await expect(added).toHaveValue("Production parity story")
          await added.fill("Edited production story")
          await expect(added).toHaveValue("Edited production story")
          await expect(
            page.getByLabel("Story card title", { exact: true })
          ).toHaveValue("")
        }
        expect(errors).toEqual([])
        observations.push(
          JSON.stringify({
            text: await page.locator("body").innerText(),
            inputs: await page
              .locator("input")
              .evaluateAll((elements) =>
                elements.map((element) => (element as HTMLInputElement).value)
              ),
          })
        )
        await page.screenshot({
          path: join(output, `${id}-${profile}.png`),
          fullPage: true,
        })
        report.push({
          id,
          profile,
          buildId: manifest.provenance.buildId,
          sourceMap: manifest.sourceMap.policy,
          errors,
        })
      } catch (error) {
        await page.screenshot({
          path: join(output, `${id}-${profile}-failure.png`),
          fullPage: true,
        })
        console.error({
          id,
          profile,
          errors,
          body: await page.locator("body").innerText(),
        })
        throw error
      } finally {
        await context.close()
        server.stop(true)
      }
    }
    expect(observations[1]).toBe(observations[0])
    console.log(
      `${id}: development/release browser interactions and visible text match`
    )
  }
  writeFileSync(
    join(output, "report.json"),
    `${JSON.stringify({ schema: 1, applications: report }, null, 2)}\n`
  )
} finally {
  await browser.close()
  rmSync(join(output, ".inputs"), { recursive: true, force: true })
}
