import type { Unit } from "./effect"
import type { Eq } from "./equality"
import type { Ord } from "./sequence"
import { Equal, Greater, Less } from "./sum"
import { fullFold, simpleFold } from "./unicode-case"
import { CATEGORY_NAMES } from "./unicode-data"
import { normalizeText } from "./unicode-normalization"
import { alphabetic, categoryIndex, whitespace } from "./unicode-properties"
import { UNICODE_VERSION } from "./unicode-version-data"

export type NormalizationForm = {
  readonly tag: "NFC" | "NFD" | "NFKC" | "NFKD"
}
export const NFC: NormalizationForm = Object.freeze({ tag: "NFC" })
export const NFD: NormalizationForm = Object.freeze({ tag: "NFD" })
export const NFKC: NormalizationForm = Object.freeze({ tag: "NFKC" })
export const NFKD: NormalizationForm = Object.freeze({ tag: "NFKD" })

export type UnicodeGeneralCategory = {
  readonly tag: (typeof CATEGORY_NAMES)[number]
}
const categories: readonly UnicodeGeneralCategory[] = CATEGORY_NAMES.map(
  (tag) => Object.freeze({ tag })
)
export const UppercaseLetter: UnicodeGeneralCategory = categories[0]!
export const LowercaseLetter: UnicodeGeneralCategory = categories[1]!
export const TitlecaseLetter: UnicodeGeneralCategory = categories[2]!
export const ModifierLetter: UnicodeGeneralCategory = categories[3]!
export const OtherLetter: UnicodeGeneralCategory = categories[4]!
export const NonspacingMark: UnicodeGeneralCategory = categories[5]!
export const SpacingMark: UnicodeGeneralCategory = categories[6]!
export const EnclosingMark: UnicodeGeneralCategory = categories[7]!
export const DecimalNumber: UnicodeGeneralCategory = categories[8]!
export const LetterNumber: UnicodeGeneralCategory = categories[9]!
export const OtherNumber: UnicodeGeneralCategory = categories[10]!
export const ConnectorPunctuation: UnicodeGeneralCategory = categories[11]!
export const DashPunctuation: UnicodeGeneralCategory = categories[12]!
export const OpenPunctuation: UnicodeGeneralCategory = categories[13]!
export const ClosePunctuation: UnicodeGeneralCategory = categories[14]!
export const InitialPunctuation: UnicodeGeneralCategory = categories[15]!
export const FinalPunctuation: UnicodeGeneralCategory = categories[16]!
export const OtherPunctuation: UnicodeGeneralCategory = categories[17]!
export const MathSymbol: UnicodeGeneralCategory = categories[18]!
export const CurrencySymbol: UnicodeGeneralCategory = categories[19]!
export const ModifierSymbol: UnicodeGeneralCategory = categories[20]!
export const OtherSymbol: UnicodeGeneralCategory = categories[21]!
export const SpaceSeparator: UnicodeGeneralCategory = categories[22]!
export const LineSeparator: UnicodeGeneralCategory = categories[23]!
export const ParagraphSeparator: UnicodeGeneralCategory = categories[24]!
export const Control: UnicodeGeneralCategory = categories[25]!
export const Format: UnicodeGeneralCategory = categories[26]!
export const PrivateUse: UnicodeGeneralCategory = categories[27]!
export const Unassigned: UnicodeGeneralCategory = categories[28]!

export const version = (_unit: Unit): string => UNICODE_VERSION
export const normalize = (form: NormalizationForm, text: string): string =>
  normalizeText(form.tag, text)
export const isNormalized = (form: NormalizationForm, text: string): boolean =>
  normalize(form, text) === text
export const generalCategory = (value: string): UnicodeGeneralCategory =>
  categories[categoryIndex(value.codePointAt(0)!)]!
export const isAlphabetic = (value: string): boolean =>
  alphabetic(value.codePointAt(0)!)
export const isWhitespace = (value: string): boolean =>
  whitespace(value.codePointAt(0)!)
export const isDecimalDigit = (value: string): boolean =>
  categoryIndex(value.codePointAt(0)!) === 8
export const isMark = (value: string): boolean => {
  const index = categoryIndex(value.codePointAt(0)!)
  return index >= 5 && index <= 7
}
export const simpleCaseFold = simpleFold
export const fullCaseFold = fullFold

export const normalizationFormEq: Eq<NormalizationForm> = Object.freeze({
  eq: (left: NormalizationForm) => (right: NormalizationForm) =>
    left.tag === right.tag,
})

export const unicodeGeneralCategoryEq: Eq<UnicodeGeneralCategory> =
  Object.freeze({
    eq: (left: UnicodeGeneralCategory) => (right: UnicodeGeneralCategory) =>
      left.tag === right.tag,
  })

/** Declaration order in spec 10.8, not host collation or localized category names. */
export const unicodeGeneralCategoryOrd: Ord<UnicodeGeneralCategory> &
  Eq<UnicodeGeneralCategory> = Object.freeze({
  ...unicodeGeneralCategoryEq,
  compare:
    (left: UnicodeGeneralCategory) => (right: UnicodeGeneralCategory) => {
      const a = CATEGORY_NAMES.indexOf(left.tag),
        b = CATEGORY_NAMES.indexOf(right.tag)
      return a < b ? Less : a > b ? Greater : Equal
    },
})
