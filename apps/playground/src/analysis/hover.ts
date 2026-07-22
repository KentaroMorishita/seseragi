import type { AnalysisDocument } from "../compiler/types"
import {
  utf16OffsetToUtf8Byte,
  utf8RangeToUtf16,
} from "../diagnostics/source-range"
import { queryAnalysisAt } from "./document"

export type EditorHover = {
  readonly from: number
  readonly to: number
  readonly title: string
  readonly type?: string
  readonly module?: string
  readonly constraints: readonly string[]
  readonly remaining: readonly string[]
  readonly description?: string
}

export function analysisHoverAt(
  analysis: AnalysisDocument | undefined,
  source: string,
  utf16Position: number
): EditorHover | undefined {
  if (analysis === undefined) return undefined
  const bytePosition = utf16OffsetToUtf8Byte(source, utf16Position)
  const query = queryAnalysisAt(analysis, bytePosition)
  if (
    query.symbol === undefined &&
    query.type === undefined &&
    query.callable === undefined
  ) {
    return undefined
  }
  const byteRange = query.range ?? query.symbol?.definition
  if (byteRange === undefined) return undefined
  const range = utf8RangeToUtf16(source, byteRange)
  const callable = query.callable
  const fallbackName = query.symbol?.name ?? "expression"
  return {
    from: range.from,
    to: Math.max(range.from + 1, range.to),
    title:
      callable?.signature ??
      `${fallbackName}: ${query.symbol?.typeName ?? query.type ?? "_"}`,
    ...(query.type === undefined ? {} : { type: query.type }),
    ...(query.symbol?.module === undefined && callable?.module === undefined
      ? {}
      : { module: callable?.module ?? query.symbol?.module }),
    constraints: callable?.constraints ?? [],
    remaining:
      callable?.remainingParameters.map((parameter) =>
        parameter.name === undefined
          ? parameter.type
          : `${parameter.name}: ${parameter.type}`
      ) ?? [],
    ...(query.symbol?.description === undefined
      ? {}
      : { description: query.symbol.description }),
  }
}
