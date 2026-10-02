import process from "node:process"

try {
  console.log(`Directory: ${process.cwd().replaceAll("\\", "/")}`)
} catch {
  console.log("Cannot read the directory")
}
