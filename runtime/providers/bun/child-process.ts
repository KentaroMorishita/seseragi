import { createChildProcessesProvider } from "../child-process"

export const provider = createChildProcessesProvider(
  "seseragi/runtime-bun#child-process",
  "bun-process"
)
