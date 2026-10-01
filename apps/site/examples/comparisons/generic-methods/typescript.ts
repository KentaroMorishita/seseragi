class Box<A> {
  constructor(readonly value: A) {}

  get(): A {
    return this.value
  }

  replace<B>(value: B): Box<B> {
    return new Box(value)
  }
}

const number = new Box(42)
const text = number.replace("ready")
console.log(`${number.get() + 1}\n${text.get()}!`)
