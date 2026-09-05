import { assertUnicodeVersion as $ssrg$assertUnicodeVersion } from "@seseragi/runtime/unicode-version"
import { domErrorShow as _ssrg_show_domErrorShow, domErrorDebug as _ssrg_debug_domErrorDebug, stringShow as _ssrg_show_stringShow, domRuntimeErrorShow as _ssrg_show_domRuntimeErrorShow, stringDebug as _ssrg_debug_stringDebug, domRuntimeErrorDebug as _ssrg_debug_domRuntimeErrorDebug, htmlBuildErrorShow as _ssrg_show_htmlBuildErrorShow, htmlBuildErrorDebug as _ssrg_debug_htmlBuildErrorDebug } from "@seseragi/runtime/show"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"
import { InvalidTagName as _ssrg_html_InvalidTagName, type HtmlBuildError as HtmlBuildError } from "@seseragi/runtime/html"
import { type DomError as DomError, type DomRuntimeError as DomRuntimeError } from "@seseragi/runtime/dom"
$ssrg$assertUnicodeVersion("17.0.0")

export const showDomError = (value: DomError) => ((_ssrg_show_domErrorShow["show"](value)) as string)
export const debugDomError = (value: DomError) => ((_ssrg_debug_domErrorDebug["debug"](value)) as string)
export const showDomRuntimeError = (value: DomRuntimeError<string>) => ((_ssrg_show_domRuntimeErrorShow<string>(_ssrg_show_stringShow)["show"](value)) as string)
export const debugDomRuntimeError = (value: DomRuntimeError<string>) => ((_ssrg_debug_domRuntimeErrorDebug<string>(_ssrg_debug_stringDebug)["debug"](value)) as string)
export const showHtmlBuildError = (value: HtmlBuildError) => ((_ssrg_show_htmlBuildErrorShow["show"](value)) as string)
export const debugHtmlBuildError = (value: HtmlBuildError) => ((_ssrg_debug_htmlBuildErrorDebug["debug"](value)) as string)
export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_console_println(((_ssrg_show_htmlBuildErrorShow["show"](_ssrg_html_InvalidTagName("1bad"))) as string)), () => _ssrg_console_println(((_ssrg_debug_htmlBuildErrorDebug["debug"](_ssrg_html_InvalidTagName("1bad"))) as string)))
