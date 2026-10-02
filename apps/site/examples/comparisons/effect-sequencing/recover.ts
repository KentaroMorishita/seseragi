type Result = { ok: true; value: string } | { ok: false; reason: string }
function lookup(key: string): Result {
  if (key === "saved") return { ok: true, value: "Release" }
  return {
    ok: false,
    reason: key === "missing" ? "missing" : "invalid setting",
  }
}
function display(key: string): void {
  let result = lookup(key)
  if (!result.ok && result.reason === "missing")
    result = { ok: true, value: "Preview" }
  console.log(
    result.ok ? `Title: ${result.value}` : `Rejected: ${result.reason}`
  )
}
display("saved")
display("missing")
display("broken")
export {}
