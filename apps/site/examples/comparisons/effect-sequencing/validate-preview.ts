type Result = { ok: true; value: string } | { ok: false; reason: string }
function checkedTitle(title: string): Result {
  return title === ""
    ? { ok: false, reason: "title is required" }
    : { ok: true, value: title }
}
function prepare(title: string, template: string | undefined): Result {
  const checked = checkedTitle(title)
  if (!checked.ok) return checked
  if (template === undefined)
    return { ok: false, reason: "template is missing" }
  return { ok: true, value: `${template}: ${checked.value}` }
}
function preview(title: string, template: string | undefined): void {
  const result = prepare(title, template)
  console.log(
    result.ok ? `Ready: ${result.value}` : `Rejected: ${result.reason}`
  )
}
preview("Search improvements", "Release")
preview("", undefined)
preview("Search improvements", undefined)
preview("Search improvements", "")
export {}
