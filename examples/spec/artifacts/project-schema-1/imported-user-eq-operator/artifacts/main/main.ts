import { Ready, Waiting, __ssrg$instance$Eq$0, type Status } from "./domain.js"
import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
const $ssrg$source106 = { file: "project/imported-user-eq-operator/src/main.ssrg", line: 5, column: 3, uri: "seseragi://fixture/imported-user-eq-operator::main", start: 106, end: 119 };
const $ssrg$source377 = { file: "project/imported-user-eq-operator/src/main.ssrg", line: 18, column: 3, uri: "seseragi://fixture/imported-user-eq-operator::main", start: 377, end: 432 };
const $ssrg$source378 = { file: "project/imported-user-eq-operator/src/main.ssrg", line: 18, column: 4, uri: "seseragi://fixture/imported-user-eq-operator::main", start: 378, end: 394 };
const $ssrg$source396 = { file: "project/imported-user-eq-operator/src/main.ssrg", line: 18, column: 22, uri: "seseragi://fixture/imported-user-eq-operator::main", start: 396, end: 419 };
$ssrg$assertUnicodeVersion("17.0.0")

const same = <T,>(left: T) => (right: T) => (__ssrg$evidence$0: import("@seseragi/runtime/equality").Eq<T>) => $ssrg$atSource($ssrg$source106, () => __ssrg$evidence$0["eq"](left)(right), false)
const different = <T,>(left: T) => (right: T) => (__ssrg$evidence$0: import("@seseragi/runtime/equality").Eq<T>) => __ssrg$evidence$0["eq"](left)(right) === false
const render = (checks: readonly [boolean, boolean]) => (($ssrg_match: readonly [boolean, boolean]): string => $ssrg_match[0] === true && $ssrg_match[1] === true ? "Imported Eq: same / different" : "unexpected imported Eq result")(checks)
export const main = (_unit: undefined) => $ssrg$atSource($ssrg$source377, () => _ssrg_console_println($ssrg$atSource($ssrg$source377, () => render([$ssrg$atSource($ssrg$source378, () => ((same<Status>(Ready)(Ready)(__ssrg$instance$Eq$0)) as boolean), false), $ssrg$atSource($ssrg$source396, () => ((different<Status>(Ready)(Waiting)(__ssrg$instance$Eq$0)) as boolean), false)] as const), false)), true)
