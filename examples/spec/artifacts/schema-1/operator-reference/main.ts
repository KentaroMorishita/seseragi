import { add as _ssrg_int64_add } from "@seseragi/runtime/int64"

export const foldPair = (step: (argument: bigint) => (argument: bigint) => bigint) => (initial: bigint) => (value: bigint) => step(initial)(value)
export const addPair = (initial: bigint) => (value: bigint) => foldPair(_ssrg_int64_add)(initial)(value)
