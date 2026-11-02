import { type Hand, Rock, Scissors, identity, decide, renderOutcome, type Outcome } from "./domain.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
const $ssrg$source266 = { file: "project/rock-paper-scissors-domain-split/src/main.ssrg", line: 10, column: 3, uri: "seseragi://fixture/rps-domain-split::main", start: 266, end: 301 };
const $ssrg$source281 = { file: "project/rock-paper-scissors-domain-split/src/main.ssrg", line: 10, column: 18, uri: "seseragi://fixture/rps-domain-split::main", start: 281, end: 300 };
const $ssrg$source350 = { file: "project/rock-paper-scissors-domain-split/src/main.ssrg", line: 13, column: 3, uri: "seseragi://fixture/rps-domain-split::main", start: 350, end: 408 };
const $ssrg$source360 = { file: "project/rock-paper-scissors-domain-split/src/main.ssrg", line: 13, column: 13, uri: "seseragi://fixture/rps-domain-split::main", start: 360, end: 407 };
const $ssrg$source375 = { file: "project/rock-paper-scissors-domain-split/src/main.ssrg", line: 13, column: 28, uri: "seseragi://fixture/rps-domain-split::main", start: 375, end: 406 };
const $ssrg$source385 = { file: "project/rock-paper-scissors-domain-split/src/main.ssrg", line: 13, column: 38, uri: "seseragi://fixture/rps-domain-split::main", start: 385, end: 405 };
$ssrg$assertUnicodeVersion("17.0.0")

export const isOpeningDraw = (first: Hand) => (second: Hand) => (($ssrg_match: readonly [Hand, Hand]): boolean => $ssrg_match[0].tag === "Rock" && $ssrg_match[1].tag === "Rock" ? true : false)([first, second] as const)
export const play = (first: Hand) => (second: Hand) => $ssrg$atSource($ssrg$source266, () => renderOutcome($ssrg$atSource($ssrg$source281, () => decide(first)(second), false)), false)
export const openingMessage = (unit: undefined) => $ssrg$atSource($ssrg$source350, () => identity<string>($ssrg$atSource($ssrg$source360, () => renderOutcome($ssrg$atSource($ssrg$source375, () => identity<Outcome>($ssrg$atSource($ssrg$source385, () => decide(Rock)(Scissors), false)), false)), false)), false)
