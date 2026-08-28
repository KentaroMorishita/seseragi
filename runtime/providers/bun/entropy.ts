import { createEntropyProvider } from "../entropy"

export const provider = createEntropyProvider(
  "seseragi/runtime-bun#entropy",
  ["bun-process"],
  {
    available: () => typeof globalThis.crypto?.getRandomValues === "function",
    fill: (values) => globalThis.crypto.getRandomValues(values),
  }
)
