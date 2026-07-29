import { add as _ssrg_int_add } from "@seseragi/runtime/int"

export const foldPair = (step: (argument: number) => (argument: number) => number) => (initial: number) => (value: number) => step(initial)(value)
export const addPair = (initial: number) => (value: number) => foldPair((_argument0) => (_argument1) => _ssrg_int_add(_argument0, _argument1))(initial)(value)
