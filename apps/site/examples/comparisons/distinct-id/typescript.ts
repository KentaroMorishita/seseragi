type UserId = { kind: "user"; value: number }
export type OrderId = { kind: "order"; value: number }

function userLabel(id: UserId): string {
  return `user: ${id.value}`
}

const user: UserId = { kind: "user", value: 42 }
console.log(userLabel(user))
