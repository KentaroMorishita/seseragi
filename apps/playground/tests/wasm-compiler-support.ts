// Loads the committed Playground WASM for the compile tests.

export type ProjectResponse = Readonly<{
  readonly status: "success" | "failure"
  readonly problems?: readonly unknown[]
  readonly diagnostics?: readonly {
    readonly diagnostics: {
      readonly diagnostics: readonly {
        readonly code: string
        readonly messageKey: string
        readonly related: readonly { readonly message: string }[]
      }[]
    }
  }[]
}>

export type FormatResponse = Readonly<{
  readonly status: "success" | "failure"
  readonly source?: string
  readonly diagnostics?: unknown
}>

type WasmBindings = Readonly<{
  readonly default: (input: {
    readonly module_or_path: ArrayBuffer
  }) => Promise<unknown>
  readonly compile_project: (request: string) => string
  readonly compile_single_file: (
    sourceName: string,
    moduleId: string,
    source: string
  ) => string
  readonly format_project_file: (request: string, path: string) => string
}>

let bindings: WasmBindings | undefined

export async function loadBindings(): Promise<WasmBindings> {
  if (bindings !== undefined) return bindings
  const bindingsUrl = new URL(
    "../src/wasm/pkg/seseragi_wasm.js",
    import.meta.url
  ).href
  bindings = (await import(bindingsUrl)) as WasmBindings
  const wasm = await Bun.file(
    new URL("../src/wasm/pkg/seseragi_wasm_bg.wasm", import.meta.url)
  ).arrayBuffer()
  await bindings.default({ module_or_path: wasm })
  return bindings
}
