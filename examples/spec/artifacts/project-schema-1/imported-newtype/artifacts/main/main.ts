import { UserId, type UserId as UserId_1 } from "./domain.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source147 = { file: "project/imported-newtype/src/main.ssrg", line: 9, column: 9, uri: "seseragi://fixture/imported-newtype::main", start: 147, end: 153 };
const $ssrg$source256 = { file: "project/imported-newtype/src/main.ssrg", line: 15, column: 3, uri: "seseragi://fixture/imported-newtype::main", start: 256, end: 270 };
$ssrg$assertUnicodeVersion("17.0.0")

const raw = (id: UserId) => (($ssrg_match: UserId): number => $ssrg_match.tag === "UserId" ? ((value: number): number => value)($ssrg_match.value) : ((): never => { throw new globalThis.Error("non-exhaustive Seseragi match"); })())(id)
const render = (id: UserId) => (($ssrg_match: number): string => $ssrg_match === 42 ? "Imported newtype: 42" : "unexpected imported newtype")($ssrg$atSource($ssrg$source147, () => raw(id), false))
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source256, () => _ssrg_console_println($ssrg$atSource($ssrg$source256, () => render($ssrg$atSource($ssrg$source256, () => UserId(42), false)), false)), true)
