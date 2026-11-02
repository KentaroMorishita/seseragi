import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { atSource as $ssrg$atSource } from "@seseragi/runtime/effect"
import { domErrorShow as _ssrg_show_domErrorShow, domErrorDebug as _ssrg_debug_domErrorDebug, stringShow as _ssrg_show_stringShow, domRuntimeErrorShow as _ssrg_show_domRuntimeErrorShow, stringDebug as _ssrg_debug_stringDebug, domRuntimeErrorDebug as _ssrg_debug_domRuntimeErrorDebug, htmlBuildErrorShow as _ssrg_show_htmlBuildErrorShow, htmlBuildErrorDebug as _ssrg_debug_htmlBuildErrorDebug } from "@seseragi/runtime/show"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { InvalidTagName as _ssrg_html_InvalidTagName, type HtmlBuildError as HtmlBuildError } from "@seseragi/runtime/html"
import { type DomError as DomError, type DomRuntimeError as DomRuntimeError } from "@seseragi/runtime/dom"
const $ssrg$source127 = { file: "main.ssrg", line: 5, column: 3, uri: "seseragi://artifact/standard-error-display", start: 127, end: 137 };
const $ssrg$source194 = { file: "main.ssrg", line: 8, column: 3, uri: "seseragi://artifact/standard-error-display", start: 194, end: 205 };
const $ssrg$source283 = { file: "main.ssrg", line: 11, column: 3, uri: "seseragi://artifact/standard-error-display", start: 283, end: 293 };
const $ssrg$source372 = { file: "main.ssrg", line: 14, column: 3, uri: "seseragi://artifact/standard-error-display", start: 372, end: 383 };
const $ssrg$source452 = { file: "main.ssrg", line: 17, column: 3, uri: "seseragi://artifact/standard-error-display", start: 452, end: 462 };
const $ssrg$source532 = { file: "main.ssrg", line: 20, column: 3, uri: "seseragi://artifact/standard-error-display", start: 532, end: 543 };
const $ssrg$source577 = { file: "main.ssrg", line: 24, column: 5, uri: "seseragi://artifact/standard-error-display", start: 577, end: 620 };
const $ssrg$source586 = { file: "main.ssrg", line: 24, column: 14, uri: "seseragi://artifact/standard-error-display", start: 586, end: 619 };
const $ssrg$source592 = { file: "main.ssrg", line: 24, column: 20, uri: "seseragi://artifact/standard-error-display", start: 592, end: 618 };
const $ssrg$source625 = { file: "main.ssrg", line: 25, column: 5, uri: "seseragi://artifact/standard-error-display", start: 625, end: 669 };
const $ssrg$source634 = { file: "main.ssrg", line: 25, column: 14, uri: "seseragi://artifact/standard-error-display", start: 634, end: 668 };
const $ssrg$source641 = { file: "main.ssrg", line: 25, column: 21, uri: "seseragi://artifact/standard-error-display", start: 641, end: 667 };
$ssrg$assertUnicodeVersion("17.0.0")

export const showDomError = (value: DomError) => $ssrg$atSource($ssrg$source127, () => ((_ssrg_show_domErrorShow["show"](value)) as string), false)
export const debugDomError = (value: DomError) => $ssrg$atSource($ssrg$source194, () => ((_ssrg_debug_domErrorDebug["debug"](value)) as string), false)
export const showDomRuntimeError = (value: DomRuntimeError<string>) => $ssrg$atSource($ssrg$source283, () => ((_ssrg_show_domRuntimeErrorShow<string>(_ssrg_show_stringShow)["show"](value)) as string), false)
export const debugDomRuntimeError = (value: DomRuntimeError<string>) => $ssrg$atSource($ssrg$source372, () => ((_ssrg_debug_domRuntimeErrorDebug<string>(_ssrg_debug_stringDebug)["debug"](value)) as string), false)
export const showHtmlBuildError = (value: HtmlBuildError) => $ssrg$atSource($ssrg$source452, () => ((_ssrg_show_htmlBuildErrorShow["show"](value)) as string), false)
export const debugHtmlBuildError = (value: HtmlBuildError) => $ssrg$atSource($ssrg$source532, () => ((_ssrg_debug_htmlBuildErrorDebug["debug"](value)) as string), false)
export const main = (_unit: undefined) => _ssrg_effect_flatMap($ssrg$atSource($ssrg$source577, () => _ssrg_console_println($ssrg$atSource($ssrg$source586, () => ((_ssrg_show_htmlBuildErrorShow["show"]($ssrg$atSource($ssrg$source592, () => _ssrg_html_InvalidTagName("1bad"), false))) as string), false)), true), () => $ssrg$atSource($ssrg$source625, () => _ssrg_console_println($ssrg$atSource($ssrg$source634, () => ((_ssrg_debug_htmlBuildErrorDebug["debug"]($ssrg$atSource($ssrg$source641, () => _ssrg_html_InvalidTagName("1bad"), false))) as string), false)), true))
