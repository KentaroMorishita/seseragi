import { readFile } from "node:fs/promises"
import { createRequire } from "node:module"
import { fileURLToPath } from "node:url"
import { build as bundleExecutionFrame } from "esbuild"
import { defineConfig, type HtmlTagDescriptor, type Plugin } from "vite"
import packageManifest from "./package.json" with { type: "json" }
import { fontPackages } from "./src/ui/design/font-packages"
import { localeBootScript } from "./src/ui/i18n"
import { themeBootScript } from "./src/ui/theme"

const appRoot = fileURLToPath(new URL(".", import.meta.url))
const publicOrigin = "https://seseragi.vercel.app"
const socialImage = `${publicOrigin}/brand/seseragi-social-preview.png`

const brandTags = (
  title: string,
  description: string,
  url: string
): HtmlTagDescriptor[] => [
  {
    tag: "link",
    attrs: { rel: "stylesheet", href: "/brand/brand.css" },
    injectTo: "head",
  },
  {
    tag: "link",
    attrs: {
      rel: "icon",
      href: "/brand/seseragi-icon.svg",
      type: "image/svg+xml",
    },
    injectTo: "head",
  },
  {
    tag: "link",
    attrs: {
      rel: "icon",
      href: "/brand/favicon-32x32.png",
      type: "image/png",
      sizes: "32x32",
    },
    injectTo: "head",
  },
  {
    tag: "link",
    attrs: {
      rel: "icon",
      href: "/brand/favicon-16x16.png",
      type: "image/png",
      sizes: "16x16",
    },
    injectTo: "head",
  },
  {
    tag: "link",
    attrs: {
      rel: "apple-touch-icon",
      href: "/brand/apple-touch-icon.png",
      sizes: "180x180",
    },
    injectTo: "head",
  },
  {
    tag: "link",
    attrs: { rel: "manifest", href: "/brand/site.webmanifest" },
    injectTo: "head",
  },
  {
    tag: "meta",
    attrs: { property: "og:type", content: "website" },
    injectTo: "head",
  },
  {
    tag: "meta",
    attrs: { property: "og:site_name", content: "Seseragi" },
    injectTo: "head",
  },
  {
    tag: "meta",
    attrs: { property: "og:title", content: title },
    injectTo: "head",
  },
  {
    tag: "meta",
    attrs: { property: "og:description", content: description },
    injectTo: "head",
  },
  {
    tag: "meta",
    attrs: { property: "og:url", content: url },
    injectTo: "head",
  },
  {
    tag: "meta",
    attrs: { property: "og:image", content: socialImage },
    injectTo: "head",
  },
  {
    tag: "meta",
    attrs: { property: "og:image:width", content: "1200" },
    injectTo: "head",
  },
  {
    tag: "meta",
    attrs: { property: "og:image:height", content: "630" },
    injectTo: "head",
  },
  {
    tag: "meta",
    attrs: { name: "twitter:card", content: "summary_large_image" },
    injectTo: "head",
  },
  {
    tag: "meta",
    attrs: { name: "twitter:title", content: title },
    injectTo: "head",
  },
  {
    tag: "meta",
    attrs: { name: "twitter:description", content: description },
    injectTo: "head",
  },
  {
    tag: "meta",
    attrs: { name: "twitter:image", content: socialImage },
    injectTo: "head",
  },
]

const brandSurfacePlugin: Plugin = {
  name: "seseragi-brand-surface",
  transformIndexHtml(html, context) {
    if (context.filename.endsWith("/runtime-preview.html")) return html
    const isTour = context.filename.endsWith("/tour/index.html")
    const isDeepDive = context.filename.endsWith("/deep-dive/index.html")
    const title = isTour
      ? "A Tour of Seseragi"
      : isDeepDive
        ? "Seseragi Deep Dive"
        : "Seseragi Playground"
    const description = isTour
      ? "現行Seseragiを編集・実行しながら順に学ぶcanonical Tour"
      : isDeepDive
        ? "Tourとは独立してSeseragiの設計と意味論を掘り下げるDeep Dive"
        : "Rust compilerと同じdriverで動くSeseragi Playground"
    const url = isTour
      ? `${publicOrigin}/tour/`
      : isDeepDive
        ? `${publicOrigin}/deep-dive/`
        : `${publicOrigin}/`

    return {
      html: html.replaceAll("__SESERAGI_VERSION__", packageManifest.version),
      tags: [
        // Apply the saved theme and URL locale before the first paint.
        { tag: "script", children: themeBootScript, injectTo: "head-prepend" },
        { tag: "script", children: localeBootScript, injectTo: "head-prepend" },
        ...brandTags(title, description, url),
      ],
    }
  },
}

function rewriteEnglishPath(url: string | undefined): string | undefined {
  if (url === undefined) return undefined
  const queryIndex = url.indexOf("?")
  const pathname = queryIndex === -1 ? url : url.slice(0, queryIndex)
  const search = queryIndex === -1 ? "" : url.slice(queryIndex)
  if (pathname === "/en" || pathname === "/en/") return `/index.html${search}`
  if (pathname === "/en/tour" || pathname === "/en/tour/") {
    return `/tour/index.html${search}`
  }
  if (pathname === "/en/deep-dive" || pathname === "/en/deep-dive/") {
    return `/deep-dive/index.html${search}`
  }
  return undefined
}

const englishRoutesPlugin: Plugin = {
  name: "seseragi-english-routes",
  enforce: "post",
  configureServer(server) {
    server.middlewares.use((req, _res, next) => {
      const rewritten = rewriteEnglishPath(req.url)
      if (rewritten !== undefined) req.url = rewritten
      next()
    })
  },
  configurePreviewServer(server) {
    server.middlewares.use((req, _res, next) => {
      const rewritten = rewriteEnglishPath(req.url)
      if (rewritten !== undefined) req.url = rewritten
      next()
    })
  },
  generateBundle(_options, bundle) {
    const pairs: readonly (readonly [string, string, string, string])[] = [
      [
        "index.html",
        "en/index.html",
        `${publicOrigin}/`,
        `${publicOrigin}/en/`,
      ],
      [
        "tour/index.html",
        "en/tour/index.html",
        `${publicOrigin}/tour/`,
        `${publicOrigin}/en/tour/`,
      ],
    ]
    for (const [sourceFile, targetFile, jaUrl, enUrl] of pairs) {
      const asset = bundle[sourceFile]
      if (asset?.type !== "asset" || typeof asset.source !== "string") {
        this.error(
          `englishRoutesPlugin expected emitted HTML asset ${sourceFile}`
        )
      }
      const localized = asset.source
        .replace('<html lang="ja">', '<html lang="en">')
        .replaceAll(`content="${jaUrl}"`, `content="${enUrl}"`)
      this.emitFile({
        type: "asset",
        fileName: targetFile,
        source: localized,
      })
    }
  },
}

// The self-hosted fonts are licensed under the SIL Open Font License 1.1,
// which asks for the license text to travel with the font files.
const fontLicensePlugin: Plugin = {
  name: "seseragi-font-licenses",
  apply: "build",
  async generateBundle() {
    const require = createRequire(import.meta.url)
    for (const font of fontPackages) {
      this.emitFile({
        type: "asset",
        fileName: font.licenseFile,
        source: await readFile(
          require.resolve(`${font.packageName}/LICENSE`),
          "utf8"
        ),
      })
    }
  },
}

// A classic self-contained bundle can run inside an opaque srcdoc without
// granting same-origin access or relaxing the dev server's CORS policy.
const isolatedExecutionPlugin: Plugin = {
  name: "seseragi-isolated-execution",
  resolveId(id) {
    if (id === "virtual:seseragi-execution-frame") return `\0${id}`
  },
  generateBundle(_options, bundle) {
    for (const output of Object.values(bundle)) {
      if (
        output.type === "chunk" &&
        Object.keys(output.modules).some((id) =>
          id.endsWith("/runtime/browser-execution.ts")
        )
      )
        this.error(
          "the program evaluator must only run inside the isolated frame"
        )
    }
  },
  async load(id) {
    if (id !== "\0virtual:seseragi-execution-frame") return
    const worker = await bundleExecutionFrame({
      absWorkingDir: appRoot,
      entryPoints: ["src/runtime/execution-worker.ts"],
      bundle: true,
      write: false,
      format: "iife",
      platform: "browser",
      target: "es2022",
      minify: true,
      metafile: true,
    })
    const result = await bundleExecutionFrame({
      absWorkingDir: appRoot,
      entryPoints: ["src/runtime/execution-frame.ts"],
      bundle: true,
      write: false,
      format: "iife",
      platform: "browser",
      target: "es2022",
      minify: true,
      metafile: true,
      define: {
        __SESERAGI_EXECUTION_WORKER__: JSON.stringify(
          worker.outputFiles![0]!.text
        ),
      },
    })
    for (const input of new Set([
      ...Object.keys(result.metafile!.inputs),
      ...Object.keys(worker.metafile!.inputs),
    ])) {
      if (!input.startsWith("(disabled):")) {
        this.addWatchFile(fileURLToPath(new URL(input, import.meta.url)))
      }
    }
    return `export default ${JSON.stringify(result.outputFiles![0]!.text)}`
  },
}

export default defineConfig({
  publicDir: fileURLToPath(
    new URL("../../assets/brand/public", import.meta.url)
  ),
  plugins: [
    brandSurfacePlugin,
    englishRoutesPlugin,
    fontLicensePlugin,
    isolatedExecutionPlugin,
  ],
  resolve: {
    dedupe: ["@seseragi/runtime"],
  },
  build: {
    target: "es2022",
    outDir: "dist",
    rollupOptions: {
      input: {
        playground: `${appRoot}index.html`,
        runtimePreview: `${appRoot}runtime-preview.html`,
        tour: `${appRoot}tour/index.html`,
        deepDive: `${appRoot}deep-dive/index.html`,
      },
      output: {
        manualChunks(id) {
          if (id.includes("/typescript/")) return "typescript"
          if (id.includes("/node_modules/@codemirror/")) return "editor"
          return undefined
        },
      },
    },
  },
})
