type Ticket = { number: number }
type Person = { name: string }

interface Label<A> {
  label(value: A): string
}

const ticketLabel: Label<Ticket> = {
  label: (value) => `ticket-${value.number}`,
}
const personLabel: Label<Person> = {
  label: (value) => value.name,
}

console.log(ticketLabel.label({ number: 42 }))
console.log(personLabel.label({ name: "Aki" }))
