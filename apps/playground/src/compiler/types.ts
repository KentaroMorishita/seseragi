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

export type DiagnosticArtifact = {
  readonly diagnostics: readonly Diagnostic[]
}

export type AnalysisParameter = {
  readonly name?: string
  readonly type: string
}

export type AnalysisCallable = {
  readonly identity: string
  readonly name: string
  readonly module: string
  readonly typeParameters: readonly string[]
  readonly parameters: readonly AnalysisParameter[]
  readonly result: string
  readonly constraints: readonly string[]
  readonly signature: string
  readonly remainingParameters: readonly AnalysisParameter[]
}

export type AnalysisSymbol = {
  readonly id: number
  readonly identity: string
  readonly name: string
  readonly module: string
  readonly namespace: string
  readonly kind: string
  readonly definition: SourceRange
  readonly typeName?: string
  readonly callable?: AnalysisCallable
  readonly description?: string
}

export type AnalysisReferenceItem = {
  readonly identity: string
  readonly name: string
  readonly module: string
  readonly category: string
  readonly kind: string
  readonly signature?: string
  readonly description: string
  readonly typeParameters: readonly string[]
  readonly constraints: readonly string[]
}

export type AnalysisDocument = {
  readonly schema: number
  readonly source: string
  readonly module: string
  readonly diagnostics: DiagnosticArtifact
  readonly symbols: readonly AnalysisSymbol[]
  readonly symbolOccurrences: readonly {
    readonly range: SourceRange
    readonly symbol: number
  }[]
  readonly typeOccurrences: readonly {
    readonly range: SourceRange
    readonly type: string
  }[]
  readonly callableOccurrences: readonly {
    readonly range: SourceRange
    readonly callable: AnalysisCallable
  }[]
  readonly standardLibrary: readonly AnalysisReferenceItem[]
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
