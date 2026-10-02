import process from "node:process"

function readSetting(name: string): string | undefined {
  if (name.length === 0 || name.includes("\0")) {
    throw new Error("Use a nonempty setting name without a NUL character")
  }
  return process.env[name]
}
for (const name of ["", "BAD\0NAME"]) {
  try {
    readSetting(name)
    console.log("Setting name accepted")
  } catch (error) {
    console.log(
      error instanceof Error ? error.message : "Cannot read the setting"
    )
  }
}
