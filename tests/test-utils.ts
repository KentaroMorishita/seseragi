import { Parser } from "../src/parser"
import { generateTypeScript } from "../src/codegen"
import { TypeInferenceSystem } from "../src/type-inference"

export function compileSeseragi(source: string): string {
  const parser = new Parser(source)
  const program = parser.parse()
  const typeInference = new TypeInferenceSystem()
  const typeResult = typeInference.infer(program)
  return generateTypeScript(program, typeResult.nodeTypeMap)
}