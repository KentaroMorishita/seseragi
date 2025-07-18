import { Parser } from "./src/parser"

const code = `type Box<T> = T`

const parser = new Parser(code)
const parseResult = parser.parse()

console.log("Type alias declaration:")
const typeAlias = parseResult.statements?.[0] as any
console.log("Name:", typeAlias.name)
console.log("Type parameters:", typeAlias.typeParameters?.map((p: any) => ({ name: p.name, kind: p.kind })))
console.log("Aliased type:", typeAlias.aliasedType)
console.log("Aliased type kind:", typeAlias.aliasedType?.kind)
if (typeAlias.aliasedType?.kind === "PrimitiveType") {
  console.log("Aliased type name:", typeAlias.aliasedType.name)
}