import { createFileSystemProvider } from "../filesystem"

export const provider = createFileSystemProvider(
  "seseragi/runtime-node#filesystem",
  "node-process"
)
