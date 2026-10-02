type Result = { ok: true; value: string } | { ok: false; reason: string }
function checkedTitle(title: string): Result {
  return title === ""
    ? { ok: false, reason: "title is required" }
    : { ok: true, value: title }
}
function preview(title: string): void {
  const result = checkedTitle(title)
  console.log(
    result.ok ? `Ready: ${result.value}` : `Rejected: preview: ${result.reason}`
  )
}
preview("Search improvements")
preview("")
export {}
