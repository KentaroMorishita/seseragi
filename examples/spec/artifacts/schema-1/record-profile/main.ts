import { Nothing as _ssrg_maybe_Nothing, Just as _ssrg_maybe_Just } from "@seseragi/runtime/sum"
import { flatMap as _ssrg_effect_flatMap } from "@seseragi/runtime/effect"
import { println as _ssrg_console_println } from "@seseragi/runtime/console"

const profile = (name: string) => (score: bigint) => ({ "name": name, "score": score } as const)
const displayName = (user: { readonly "name": string }) => (user)["name"]
const renderProfile = (user: { readonly "label"?: string; readonly "name": string }) => (($ssrg_match: { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: string }): string => $ssrg_match.tag === "Nothing" ? "Record profile: " + displayName(user) : $ssrg_match.tag === "Just" ? ((label: string): string => label + ": " + displayName(user))($ssrg_match.value) : ((): never => { throw new Error("non-exhaustive Seseragi match"); })())((($ssrg_record) => Object.prototype.hasOwnProperty.call($ssrg_record, "label") ? _ssrg_maybe_Just($ssrg_record["label"]) : _ssrg_maybe_Nothing)(user))
export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_console_println(renderProfile(profile("Mio")(42n))), () => _ssrg_console_println(renderProfile(({ "label": "Player", "name": "Mio" } as const))))
