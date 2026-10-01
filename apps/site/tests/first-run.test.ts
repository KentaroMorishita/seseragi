import { expect, setDefaultTimeout, test } from "bun:test"
import { spawnSync } from "node:child_process"
import {
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs"
import { tmpdir } from "node:os"
import { dirname, join, relative, resolve } from "node:path"
import { sourceFromPlaygroundUrl } from "../../playground/src/workspace/source-link"
import { canonicalExample } from "../scripts/canonical-example"

const root = resolve(import.meta.dir, "../../..")
const sourceRoot = join(root, "apps/site/src")
const cli = resolve(
  root,
  process.env.SESERAGI_FIRST_RUN_BIN ??
    process.env.SESERAGI_BIN ??
    "target/debug/seseragi"
)
const source = readFileSync(
  join(root, "examples/samples/hello-world/main.ssrg"),
  "utf8"
)
const example = canonicalExample(
  "hello-world",
  "examples/samples/hello-world/main.ssrg",
  "https://seseragi.vercel.app/"
)
setDefaultTimeout(120_000)

function checked(command: string, args: string[], cwd: string) {
  const result = spawnSync(command, args, {
    cwd,
    encoding: "utf8",
    timeout: 90_000,
  })
  expect(result.status, result.stderr || result.error?.message).toBe(0)
  return result.stdout
}

function text(html: string) {
  return html
    .replace(/<[^>]+>/gu, "")
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&quot;", '"')
    .replaceAll("&#39;", "'")
    .replaceAll("&amp;", "&")
}

// Compile the production page and renderDocument with just their import
// closure. This is a focused render, not a replacement for the full site gate.
function renderPages(directory: string): Record<string, string> {
  const copied = new Set<string>()
  function copyModule(path: string) {
    if (copied.has(path)) return
    expect(path.startsWith(`${sourceRoot}/`)).toBe(true)
    copied.add(path)
    const code = readFileSync(path, "utf8")
    const destination = join(directory, "src", relative(sourceRoot, path))
    mkdirSync(dirname(destination), { recursive: true })
    copyFileSync(path, destination)
    for (const [, specifier] of code.matchAll(/from\s+"(\.[^"]+)"/gu)) {
      copyModule(
        resolve(
          dirname(path),
          specifier.endsWith(".ssrg") ? specifier : `${specifier}.ssrg`
        )
      )
    }
  }
  for (const module of [
    "pages/docs/first-run/page",
    "pages/docs/overview/page",
    "render/document",
  ]) {
    copyModule(join(sourceRoot, `${module}.ssrg`))
  }
  writeFileSync(
    join(directory, "seseragi.toml"),
    '[package]\nname = "first-run-verification"\nversion = "0.0.0"\nlanguage = ">=0.1.0 <0.2.0"\n\n[run]\nentry = "verify"\ntarget = "process"\n'
  )
  writeFileSync(
    join(directory, "src/verify.ssrg"),
    `import * as json from "std/json"
import { BuildInput, ExampleSource, HighlightPart } from "./model/build"
import { En, Ja, localizedRoute } from "./model/locale"
import { SiteCatalog } from "./model/page"
import { page as firstRun } from "./pages/docs/first-run/page"
import { page as overview } from "./pages/docs/overview/page"
import { renderDocument } from "./render/document"

struct Output deriving JsonEncode { route: String, html: String }

pub effect fn main = {
  let first = firstRun ()
  let docs = overview ()
  let input = BuildInput {
    schema: 1,
    origin: "https://seseragi.example",
    playgroundUrl: "https://seseragi.vercel.app/",
    tourUrl: "https://seseragi.vercel.app/tour/",
    grammar: "",
    examples: [ExampleSource {
      id: "hello-world",
      sourcePath: "examples/samples/hello-world/main.ssrg",
      source: ${JSON.stringify(source)},
      sha256: ${JSON.stringify(example.sha256)},
      playgroundUrl: ${JSON.stringify(example.playgroundUrl)},
      highlighted: [${example.highlighted.map((part) => `HighlightPart { text: ${JSON.stringify(part.text)}, className: ${JSON.stringify(part.className)} }`).join(",")}]
    }],
    referenceModules: []
  }
  let site = SiteCatalog { home: first, areas: [], pages: [first, docs] }
  println (json.encodeString [Output {
    route: localizedRoute locale page.path,
    html: renderDocument input site locale page
  } | locale <- [En, Ja], page <- site.pages])
}
`
  )
  checked(cli, ["lock", "update", directory], directory)
  const pages = JSON.parse(
    checked(cli, ["run", directory], directory)
  ) as Array<{
    route: string
    html: string
  }>
  return Object.fromEntries(pages.map(({ route, html }) => [route, html]))
}

function terminals(html: string) {
  return [
    ...html.matchAll(
      /<section class="code-panel terminal-panel">[\s\S]*?<pre><code>([\s\S]*?)<\/code><\/pre><\/section>/gu
    ),
  ].map((match) => text(match[1]))
}

test.skipIf(process.platform !== "linux")(
  "first-run renders equal commands in both locales and executes its exact example independently on Linux",
  () => {
    const temporary = mkdtempSync(join(tmpdir(), "seseragi-first-run-test-"))
    try {
      const generator = join(temporary, "generator")
      mkdirSync(generator)
      const pages = renderPages(generator)
      if (process.env.SITE_FIRST_RUN_OUTPUT) {
        const output = resolve(root, process.env.SITE_FIRST_RUN_OUTPUT)
        for (const [route, html] of Object.entries(pages)) {
          const file = join(output, route.slice(1), "index.html")
          mkdirSync(dirname(file), { recursive: true })
          writeFileSync(file, html)
        }
      }
      expect(Object.keys(pages)).toEqual([
        "/docs/first-run/",
        "/docs/",
        "/ja/docs/first-run/",
        "/ja/docs/",
      ])
      const en = pages["/docs/first-run/"]
      const ja = pages["/ja/docs/first-run/"]
      const commands = terminals(en)
      expect(terminals(ja)).toEqual(commands)
      expect(commands).toHaveLength(15)
      expect(commands[1]).toContain('bash -s "bun-v1.3.9"')
      expect(commands[2]).toContain("version=0.61.19")
      expect(commands[2]).toContain('sha256sum -c "$archive.sha256" &&\ntar')
      expect(commands[3]).toContain("release, commit a8641b5a81a4")
      expect(commands[6]).toContain("target=darwin-arm64")
      expect(commands[6]).toContain('shasum -a 256 -c "$archive.sha256" &&')
      expect(commands[7]).toContain("-Version 1.3.9")
      expect(commands[7]).toContain("win32-x64.zip")
      expect(commands[7]).toContain("if ($actual -eq $expected)")
      expect(commands[9]).toContain(
        "New-Item -ItemType Directory hello-seseragi"
      )
      expect(commands[14]).toContain("Get-Command seseragi, bun")
      expect(commands[10]).toBe("seseragi lint main.ssrg")
      expect(commands[11]).toBe("seseragi run main.ssrg")
      expect(commands[12]).toBe("Hello, Seseragi!")
      for (const [prefix, html, title] of [
        ["", en, "Run your first program"],
        ["/ja", ja, "最初のプログラムを実行する"],
      ]) {
        expect(html).toContain(`<h1>${title}</h1>`)
        expect(html).not.toContain("site-build-error")
        expect(html).not.toContain("docs-sidebar")
        expect(text(html)).toContain("glibc 2.34")
        expect(text(html)).toContain("Debian 13")
        expect(text(html)).toContain("darwin-arm64")
        expect(text(html)).toContain("win32-x64")
        expect(text(html)).toContain("SES-T0101")
        expect(pages[`${prefix}/docs/`]).toContain(
          `href="${prefix}/docs/first-run/"`
        )
        for (const destination of [
          "model/immutable-by-default",
          "types/annotations-and-inference",
          "syntax/function-application",
        ]) {
          expect(html).toContain(
            `href="${prefix}/docs/language/${destination}/"`
          )
        }
      }
      expect(en).toContain('href="/ja/docs/first-run/"')
      expect(ja).toContain('href="/docs/first-run/"')
      const example = en.match(
        /<code class="seseragi-highlight">([\s\S]*?)<\/code>/u
      )?.[1]
      expect(example).toBeDefined()
      const displayedSource = text(example ?? "")
      expect(displayedSource).toBe(source)
      const playgroundLink = en.match(
        /<a\b(?=[^>]*\bclass="playground-link")[^>]*\bhref="([^"]+)"/u
      )?.[1]
      expect(playgroundLink).toBeDefined()
      expect(sourceFromPlaygroundUrl(text(playgroundLink ?? ""))).toBe(source)

      const home = join(temporary, "home")
      const bin = join(home, ".local/opt/seseragi/0.61.19")
      const bunBin = join(home, ".bun/bin")
      const runtimeTmp = join(temporary, "runtime-tmp")
      for (const path of [home, bin, bunBin, runtimeTmp])
        mkdirSync(path, { recursive: true })
      copyFileSync(cli, join(bin, "seseragi"))
      copyFileSync(process.execPath, join(bunBin, "bun"))
      const env = {
        HOME: home,
        TMPDIR: runtimeTmp,
        PATH: `${bin}:${bunBin}:/usr/bin:/bin`,
      }
      const create = spawnSync(
        "/bin/bash",
        ["--noprofile", "--norc", "-c", commands[8]],
        {
          cwd: home,
          env,
          encoding: "utf8",
        }
      )
      expect(create.status, create.stderr).toBe(0)
      const project = join(home, "hello-seseragi")
      writeFileSync(join(project, "main.ssrg"), displayedSource)
      for (const [command, expected] of [
        [commands[10], ""],
        [commands[11], `${commands[12]}\n`],
      ]) {
        const result = spawnSync(
          "/bin/bash",
          ["--noprofile", "--norc", "-c", command],
          {
            cwd: project,
            env,
            encoding: "utf8",
            timeout: 30_000,
          }
        )
        expect(result.status, result.stderr).toBe(0)
        expect(result.stdout).toBe(expected)
        expect(result.stderr).toBe("")
      }
      const missingBun = spawnSync(cli, ["run", "main.ssrg"], {
        cwd: project,
        env: { ...env, PATH: bin },
        encoding: "utf8",
      })
      expect(missingBun.status).toBe(2)
      expect(missingBun.stderr).toContain("failed to launch Bun target adapter")
      const missingFile = spawnSync(cli, ["lint", "missing.ssrg"], {
        cwd: project,
        env,
        encoding: "utf8",
      })
      expect(missingFile.status).toBe(2)
      expect(missingFile.stderr).toContain("lint path does not exist")
      writeFileSync(join(project, "main.ssrg.txt"), displayedSource)
      const wrongExtension = spawnSync(cli, ["lint", "main.ssrg.txt"], {
        cwd: project,
        env,
        encoding: "utf8",
      })
      expect(wrongExtension.status).toBe(2)
      expect(wrongExtension.stderr).toContain(".ssrg source file")
      writeFileSync(
        join(project, "main.ssrg"),
        'let message: Int = "Hello, Seseragi!"\npub effect fn main = println message\n'
      )
      const mismatch = spawnSync(cli, ["lint", "main.ssrg"], {
        cwd: project,
        env,
        encoding: "utf8",
      })
      expect(mismatch.status).toBe(2)
      expect(mismatch.stderr).toContain("SES-T0101")
      expect(mismatch.stderr).toContain("expected: Int")
      expect(mismatch.stderr).toContain("actual: String")
      writeFileSync(
        join(project, "main.ssrg"),
        displayedSource.replace("pub ", "")
      )
      const privateMain = spawnSync(cli, ["run", "main.ssrg"], {
        cwd: project,
        env,
        encoding: "utf8",
      })
      expect(privateMain.status).toBe(2)
      expect(privateMain.stderr).toContain("`main` must be public")
    } finally {
      rmSync(temporary, { recursive: true, force: true })
    }
  }
)
