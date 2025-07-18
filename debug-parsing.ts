import { Parser } from "./src/parser"

const code = `type Box<T> = T
let x: Box<Int> = 42`

const parser = new Parser(code)
const parseResult = parser.parse()

console.log("Statements:", parseResult.statements?.length)
parseResult.statements?.forEach((stmt, i) => {
  console.log(`Statement ${i}:`, stmt.kind)
  if (stmt.kind === "VariableDeclaration") {
    const varDecl = stmt as any
    console.log("  Variable name:", varDecl.name)
    console.log("  Variable type:", varDecl.type)
    console.log("  Variable type kind:", varDecl.type?.kind)
    if (varDecl.type?.kind === "GenericType") {
      console.log("  Generic type name:", varDecl.type.name)
      console.log("  Type arguments:", varDecl.type.typeArguments)
    }
  }
})