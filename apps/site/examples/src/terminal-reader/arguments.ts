import process from "node:process"

const args = process.argv.slice(2)
if (args.length === 0) {
  console.log("No arguments")
} else {
  for (const value of args) console.log(`Argument: [${value}]`)
}
