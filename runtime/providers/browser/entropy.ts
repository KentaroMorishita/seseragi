import { createEntropyProvider } from "../entropy"

export const provider = createEntropyProvider(
  "seseragi/runtime-browser#entropy",
  ["browser"],
  {
    available: () => typeof globalThis.crypto?.getRandomValues === "function",
    fill: (values) => globalThis.crypto.getRandomValues(values),
  }
)
