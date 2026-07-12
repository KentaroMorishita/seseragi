import { type Hand as domain_Hand, identity as domain_identity } from "./domain.js"

export const keep = (value: domain_Hand) => value
export const run = (value: string) => domain_identity(value)
