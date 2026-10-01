type Result = { ok: true; count: number } | { ok: false; reason: string }
type Notification =
  | { kind: "count"; count: number }
  | { kind: "reason"; reason: string }
function adapt(result: Result): Notification {
  return result.ok
    ? { kind: "count", count: result.count }
    : { kind: "reason", reason: result.reason }
}
function notification(value: Notification): string {
  return value.kind === "count"
    ? `Count: ${value.count}`
    : `Reason: ${value.reason}`
}
const accepted: Result = { ok: true, count: 2 }
const rejected: Result = { ok: false, reason: "seats must be positive" }
console.log(notification(adapt(accepted)))
console.log(notification(adapt(rejected)))
export {}
