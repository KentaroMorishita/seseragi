type LoopControl = "continue" | "break"
function describe(control: LoopControl): string {
  return control === "continue"
    ? "Continue permits the next item"
    : "Break stops successfully"
}
const keepGoing: LoopControl = "continue"
const stopHere: LoopControl = "break"
console.log(describe(keepGoing))
console.log(describe(stopHere))
const stop = () => stopHere
console.log(describe(stop()))
export {}
