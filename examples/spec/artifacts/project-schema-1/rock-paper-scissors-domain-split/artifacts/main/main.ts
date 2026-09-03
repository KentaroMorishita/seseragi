import { type Hand, Rock, Scissors, identity, decide, renderOutcome, type Outcome } from "./domain.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
$ssrg$assertUnicodeVersion("17.0.0")

export const isOpeningDraw = (first: Hand) => (second: Hand) => (($ssrg_match: readonly [Hand, Hand]): boolean => $ssrg_match[0].tag === "Rock" && $ssrg_match[1].tag === "Rock" ? true : false)([first, second] as const)
export const play = (first: Hand) => (second: Hand) => renderOutcome(decide(first)(second))
export const openingMessage = (unit: undefined) => identity(renderOutcome(identity(decide(Rock)(Scissors))))
