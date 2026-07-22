export type SourceRange = {
  readonly start: number
  readonly end: number
}

export type Diagnostic = {
  readonly code: string
  readonly messageKey: string
  readonly message: string
  readonly severity?:
    | "Error"
    | "Warning"
    | "Information"
    | "Hint"
    | "error"
    | "warning"
    | "information"
    | "hint"
  readonly primary: SourceRange
  readonly related: readonly DiagnosticLabel[]
  readonly labels: readonly DiagnosticLabel[]
  readonly notes: readonly string[]
  readonly helps: readonly string[]
  readonly fixes: readonly DiagnosticFix[]
  readonly expectedType: string | null
  readonly actualType: string | null
}

export type DiagnosticLabel = {
  readonly message: string
  readonly primary: SourceRange
}

export type DiagnosticFix = {
  readonly title: string
  readonly edits: readonly {
    readonly range: SourceRange
    readonly replacement: string
  }[]
}

export type EntryContract = {
  readonly environment: readonly {
    readonly field: string
    readonly service: "console" | "stdin" | "dom"
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
