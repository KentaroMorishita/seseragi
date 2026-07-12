import { identity as domain_identity } from "./domain.js"

export const run = (value: string) => domain_identity(value)
