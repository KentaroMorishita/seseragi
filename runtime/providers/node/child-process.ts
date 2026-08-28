import { createChildProcessesProvider } from "../child-process"

export const provider = createChildProcessesProvider(
  "seseragi/runtime-node#child-process",
  "node-process"
)
