import { println as _ssrg_console_println } from "@seseragi/runtime/console"

const profile = (name: string) => (score: bigint) => ({ "name": name, "score": score } as const)
const displayName = (user: { readonly "name": string }) => (user)["name"]
const renderProfile = (user: { readonly "name": string }) => "Record profile: " + displayName(user)
export const main = (_unit: undefined) => _ssrg_console_println(renderProfile(profile("Mio")(42n)))
