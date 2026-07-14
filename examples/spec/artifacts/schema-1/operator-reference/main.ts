import { add as _ssrg_int64_add } from "@seseragi/runtime/int64"

export const foldPair = (step: (argument: bigint) => (argument: bigint) => bigint) => (initial: bigint) => (value: bigint) => step(initial)(value)
export const addPair = (initial: bigint) => (value: bigint) => foldPair((_argument0) => (_argument1) => _ssrg_int64_add(_argument0, _argument1))(initial)(value)
