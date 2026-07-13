export type SourceRange = {
  readonly start: number
  readonly end: number
}

export type Diagnostic = {
  readonly code: string
  readonly messageKey: string
  readonly severity?: "error" | "warning" | "information" | "hint"
  readonly primary: SourceRange
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
      readonly schema: number
      readonly diagnostics: DiagnosticArtifact
      readonly generated: GeneratedBundle
      readonly entry?: EntryContract
      readonly entryError?: string
    }
  | {
      readonly status: "failure"
      readonly schema: number
      readonly diagnostics: DiagnosticArtifact
    }
