/** The same binding settings and host package metadata accepted by the CLI. */
export interface BindingConversionRequest {
  schema: 1
  /** Opaque workspace snapshot key; the converter echoes it unchanged. */
  revision: string
  manifest: string
  files: { path: string; source: string }[]
  entry?: string
}

export interface GeneratedBinding {
  entry: string
  declaration: string
  output: string
  source: string
  metadata: string
  report: string
}

export interface BindingDiagnostic {
  entry?: string
  code: string
  severity: "error" | "warning"
  message: string
  path: string
  start?: number
  end?: number
  symbol?: string
}

export interface BindingConversionResponse {
  schema: 1
  revision: string
  status: "success" | "failure"
  /** Empty on every failure, including a failure after earlier entries succeed. */
  generated: GeneratedBinding[]
  diagnostics: BindingDiagnostic[]
}
