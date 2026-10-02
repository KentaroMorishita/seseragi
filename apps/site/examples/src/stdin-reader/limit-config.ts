import { stdout } from "node:process"

function describe(bytes: number): string {
  if (!Number.isSafeInteger(bytes)) return `Use a safe whole number: ${bytes}`
  if (bytes <= 0) return `Use at least 1 byte: ${bytes}`
  if (bytes > 67_108_864) return `Use at most 67108864 bytes: ${bytes}`
  return "Accepted"
}
for (const bytes of [0, -1, 1, 67_108_864, 67_108_865])
  stdout.write(`${describe(bytes)}\n`)
