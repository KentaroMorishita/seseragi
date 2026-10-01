type Delivery = { kind: "pending" } | { kind: "shipped"; tracking: string }

function label(delivery: Delivery): string {
  switch (delivery.kind) {
    case "pending":
      return "not shipped"
    case "shipped":
      return `tracking: ${delivery.tracking}`
  }
}

console.log(label({ kind: "pending" }))
console.log(label({ kind: "shipped", tracking: "JP42" }))
