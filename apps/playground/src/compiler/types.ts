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
  readonly typeDifference?: TypeDifference
}

export type TypeDifference = {
  readonly expectedType: string
  readonly actualType: string
  readonly entries: readonly TypeDifferenceEntry[]
}

export type TypeDifferenceEntry = {
  readonly path: readonly TypeDifferencePathSegment[]
  readonly kind:
    | "type-mismatch"
    | "missing-record-field"
    | "extra-record-field"
    | "field-optionality"
    | "missing-function-parameter"
    | "extra-function-parameter"
  readonly message: string
  readonly expectedType: string | null
  readonly actualType: string | null
}

export type TypeDifferencePathSegment =
  | { readonly kind: "record-field"; readonly name: string }
  | { readonly kind: "function-parameter"; readonly index: number }
  | { readonly kind: "function-result" }
  | {
      readonly kind: "type-argument"
      readonly name: string
      readonly index: number
    }
  | { readonly kind: "tuple-element"; readonly index: number }

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
    readonly service: "console" | "stdin" | "dom" | "navigation"
  }[]
  readonly failureRenderer:
    | { readonly kind: "never" }
    | {
        readonly kind: "show"
        readonly module: string
        readonly export: string
        readonly arguments?: readonly DisplayDictionaryContract[]
      }
  readonly providers?: readonly BrowserProviderSelection[]
}

export type BrowserProviderSelection = {
  readonly provider: string
  readonly service: string
  readonly target: "browser"
  readonly entryModule: string
  readonly entryExport: string
}

export type DisplayDictionaryContract = {
  readonly module: string
  readonly export: string
  readonly arguments?: readonly DisplayDictionaryContract[]
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
  readonly completionContexts?: readonly {
    readonly range: SourceRange
    readonly type: string
    readonly recordFields?: readonly {
      readonly name: string
      readonly optional: boolean
      readonly type: string
    }[]
  }[]
  readonly standardLibrary: readonly AnalysisReferenceItem[]
}

export type GeneratedBundle = {
  readonly typescript: string
}

export type ProjectRequest = {
  readonly schema: 1
  readonly manifest?: string
  /** @deprecated Compatibility fallback for manifestless single-file clients. */
  readonly entry?: string
  readonly files: readonly {
    readonly path: string
    readonly source: string
  }[]
  readonly provider?: ProjectProviderRequest
}

export type ProjectProviderTrace = {
  readonly package: string
  readonly module: string
  readonly source: string
  readonly start: number
  readonly end: number
}

export type ProjectProviderRequest = {
  readonly target: string
  readonly backendFamily: string
  readonly backendAbiMajor: number
  readonly runtimeFeatures?: readonly string[]
  readonly explicit?: Readonly<Record<string, string>>
  readonly defaults?: Readonly<Record<string, string>>
  readonly contracts?: readonly Readonly<Record<string, unknown>>[]
  readonly candidates?: readonly {
    readonly manifest: Readonly<Record<string, unknown>>
    readonly contract: Readonly<Record<string, unknown>>
    readonly visibility: "toolchain-builtin" | "root-direct-dependency"
    readonly package: {
      readonly version: string
      readonly sourceIdentity: string
      readonly contentDigest: string
    }
    readonly artifactDigest: string
    readonly hostPackages?: readonly {
      readonly name: string
      readonly version: string
      readonly sourceIdentity: string
      readonly contentDigest: string
    }[]
  }[]
  readonly transitiveRequirements?: readonly {
    readonly field: string
    readonly service: string
    readonly contractVersion: { readonly major: number; readonly minor: number }
    readonly traces: readonly ProjectProviderTrace[]
  }[]
  readonly compatibility?: {
    readonly targetExtensions?: readonly {
      readonly extension: string
      readonly trace: ProjectProviderTrace
    }[]
    readonly runtimePackages?: readonly {
      readonly provider: string
      readonly requiredIdentity: string
      readonly requiredDigest: string
      readonly actualIdentity: string
      readonly actualDigest: string
      readonly trace: ProjectProviderTrace
    }[]
    readonly compilerFeatures?: readonly {
      readonly provider: string
      readonly required: readonly string[]
      readonly supported: readonly string[]
      readonly trace: ProjectProviderTrace
    }[]
    readonly conformance?: readonly {
      readonly provider: string
      readonly requiredProfile: string
      readonly requiredDigest: string
      readonly actualProfile?: string
      readonly actualDigest?: string
      readonly trace: ProjectProviderTrace
    }[]
  }
}

export type ProjectProblem = {
  readonly code: string
  readonly message: string
  readonly path?: string
  readonly primary?: SourceRange
  readonly label?: string
  readonly details?: {
    readonly service?: string
    readonly target?: string
    readonly backendFamily?: string
    readonly backendAbiMajor?: number
    readonly provider?: string
    readonly candidates?: readonly string[]
    readonly compatibleTargets?: readonly string[]
    readonly reasons?: readonly string[]
    readonly required?: readonly string[]
    readonly actual?: readonly string[]
  }
}

export type ProjectFileDiagnostics = {
  readonly path: string
  readonly diagnostics: DiagnosticArtifact
}

export type ProjectCompileResponse =
  | {
      readonly status: "success"
      readonly schema: number
      readonly diagnostics: readonly ProjectFileDiagnostics[]
      readonly modules: readonly {
        readonly path: string
        readonly module: string
        readonly generated: GeneratedBundle
      }[]
      readonly entry: {
        readonly path: string
        readonly module: string
        readonly contract?: EntryContract
        readonly error?: string
      }
    }
  | {
      readonly status: "failure"
      readonly schema: number
      readonly diagnostics: readonly ProjectFileDiagnostics[]
      readonly problems: readonly ProjectProblem[]
    }

export type ProjectAnalysisResponse =
  | {
      readonly status: "success"
      readonly schema: number
      readonly documents: readonly {
        readonly path: string
        readonly module: string
        readonly document: AnalysisDocument
      }[]
    }
  | {
      readonly status: "failure"
      readonly schema: number
      readonly diagnostics: readonly ProjectFileDiagnostics[]
      readonly problems: readonly ProjectProblem[]
    }

export type FormatOptions = Readonly<{
  readonly lineWidth: number
}>

export type ProjectFormatResponse =
  | {
      readonly status: "success"
      readonly schema: number
      readonly path: string
      readonly source: string
      readonly changed: boolean
    }
  | {
      readonly status: "failure"
      readonly schema: number
      readonly diagnostics: readonly ProjectFileDiagnostics[]
      readonly problems: readonly ProjectProblem[]
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

export type FormatResponse =
  | {
      readonly status: "success"
      readonly schema: number
      readonly source: string
      readonly changed: boolean
    }
  | {
      readonly status: "failure"
      readonly schema: number
      readonly diagnostics: DiagnosticArtifact
    }
