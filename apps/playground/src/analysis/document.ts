import type {
  AnalysisCallable,
  AnalysisDocument,
  AnalysisSymbol,
  SourceRange,
} from "../compiler/types"

export type AnalysisQuery = {
  readonly symbol?: AnalysisSymbol
  readonly type?: string
  readonly callable?: AnalysisCallable
  readonly range?: SourceRange
}

export function queryAnalysisAt(
  analysis: AnalysisDocument,
  bytePosition: number
): AnalysisQuery {
  const symbolOccurrence = smallestContaining(
    analysis.symbolOccurrences,
    bytePosition,
    (item) => item.range
  )
  const symbol =
    symbolOccurrence === undefined
      ? undefined
      : analysis.symbols[symbolOccurrence.symbol]
  const typeOccurrence = smallestContaining(
    analysis.typeOccurrences,
    bytePosition,
    (item) => item.range
  )
  const callableOccurrence = smallestContaining(
    analysis.callableOccurrences,
    bytePosition,
    (item) => item.range
  )
  return {
    ...(symbol === undefined ? {} : { symbol }),
    ...(typeOccurrence === undefined ? {} : { type: typeOccurrence.type }),
    ...(callableOccurrence?.callable === undefined && symbol?.callable === undefined
      ? {}
      : { callable: callableOccurrence?.callable ?? symbol?.callable }),
    ...(symbolOccurrence === undefined && typeOccurrence === undefined
      ? {}
      : { range: symbolOccurrence?.range ?? typeOccurrence?.range }),
  }
}

function smallestContaining<T>(
  items: readonly T[],
  position: number,
  rangeOf: (item: T) => SourceRange
): T | undefined {
  return items
    .filter((item) => contains(rangeOf(item), position))
    .sort((left, right) => {
      const leftRange = rangeOf(left)
      const rightRange = rangeOf(right)
      return (
        leftRange.end -
        leftRange.start -
        (rightRange.end - rightRange.start)
      )
    })[0]
}

function contains(range: SourceRange, position: number): boolean {
  return range.start <= position && position < range.end
}
