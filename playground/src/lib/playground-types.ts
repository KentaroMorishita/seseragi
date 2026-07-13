export type Diagnostic = {
  readonly code: string
  readonly messageKey: string
  readonly primary: { readonly start: number; readonly end: number }
}

export type EntryContract = {
  readonly environment: readonly {
    readonly field: string
    readonly service: "console" | "stdin"
  }[]
  readonly failureRenderer:
    | { readonly kind: "never" }
    | {
        readonly kind: "show"
        readonly module: string
        readonly export: string
      }
}

type DiagnosticArtifact = {
  readonly diagnostics: readonly Diagnostic[]
}

type GeneratedBundle = {
  readonly typescript: string
}

export type CompileResponse =
  | {
      readonly status: "success"
      readonly diagnostics: DiagnosticArtifact
      readonly generated: GeneratedBundle
      readonly entry?: EntryContract
      readonly entryError?: string
    }
  | {
      readonly status: "failure"
      readonly diagnostics: DiagnosticArtifact
    }
