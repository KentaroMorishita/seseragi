import * as AST from "../src/ast"
import { generateTypeScript } from "../src/codegen"
import { Parser } from "../src/parser"
import { TypeInferenceSystem } from "../src/type-inference"

export function compileSeseragi(source: string): string {
  const parser = new Parser(source)
  const parseResult = parser.parse()
  if (!parseResult.statements) return ""
  const program = new AST.Program(parseResult.statements)
  const typeInference = new TypeInferenceSystem()
  const typeResult = typeInference.infer(program)
  return generateTypeScript(parseResult.statements, {
    typeInferenceResult: typeResult,
  })
}
