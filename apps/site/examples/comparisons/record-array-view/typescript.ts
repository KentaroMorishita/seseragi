type Named = { name: string }

function nameOf(value: Named): string {
  return value.name
}

const user = { name: "Aki", id: 1 }
const users = [user]
const names: Named[] = users.map(({ name }) => ({ name }))
console.log(`${nameOf(user)}\n[${names.map(nameOf).join(", ")}]`)
