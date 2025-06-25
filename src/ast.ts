/**
 * Abstract Syntax Tree (AST) Node Definitions for Seseragi
 */

// Base AST Node
export abstract class ASTNode {
  abstract kind: string
  line: number
  column: number

  constructor(line: number, column: number) {
    this.line = line
    this.column = column
  }
}

// =============================================================================
// Type System
// =============================================================================

export abstract class Type extends ASTNode {
  abstract name: string
}

export class PrimitiveType extends Type {
  kind = "PrimitiveType"
  name: string

  constructor(name: string, line: number, column: number) {
    super(line, column)
    this.name = name
  }
}

export class FunctionType extends Type {
  kind = "FunctionType"
  name = "Function"
  paramType: Type
  returnType: Type

  constructor(paramType: Type, returnType: Type, line: number, column: number) {
    super(line, column)
    this.paramType = paramType
    this.returnType = returnType
  }
}

export class GenericType extends Type {
  kind = "GenericType"
  name: string
  typeArguments: Type[]

  constructor(
    name: string,
    typeArguments: Type[],
    line: number,
    column: number
  ) {
    super(line, column)
    this.name = name
    this.typeArguments = typeArguments
  }
}

// =============================================================================
// Expressions
// =============================================================================

export abstract class Expression extends ASTNode {
  type?: Type // For type checking
}

export class Literal extends Expression {
  kind = "Literal"
  value: string | number | boolean
  literalType: "string" | "integer" | "float" | "boolean"

  constructor(
    value: string | number | boolean,
    literalType: "string" | "integer" | "float" | "boolean",
    line: number,
    column: number
  ) {
    super(line, column)
    this.value = value
    this.literalType = literalType
  }
}

export class Identifier extends Expression {
  kind = "Identifier"
  name: string

  constructor(name: string, line: number, column: number) {
    super(line, column)
    this.name = name
  }
}

export class BinaryOperation extends Expression {
  kind = "BinaryOperation"
  left: Expression
  operator: string
  right: Expression

  constructor(
    left: Expression,
    operator: string,
    right: Expression,
    line: number,
    column: number
  ) {
    super(line, column)
    this.left = left
    this.operator = operator
    this.right = right
  }
}

export class UnaryOperation extends Expression {
  kind = "UnaryOperation"
  operator: string
  operand: Expression

  constructor(
    operator: string,
    operand: Expression,
    line: number,
    column: number
  ) {
    super(line, column)
    this.operator = operator
    this.operand = operand
  }
}

export class FunctionCall extends Expression {
  kind = "FunctionCall"
  function: Expression
  arguments: Expression[]

  constructor(
    func: Expression,
    args: Expression[],
    line: number,
    column: number
  ) {
    super(line, column)
    this.function = func
    this.arguments = args
  }
}

export class BuiltinFunctionCall extends Expression {
  kind = "BuiltinFunctionCall"
  functionName: "print" | "putStrLn" | "toString"
  arguments: Expression[]

  constructor(
    functionName: "print" | "putStrLn" | "toString",
    args: Expression[],
    line: number,
    column: number
  ) {
    super(line, column)
    this.functionName = functionName
    this.arguments = args
  }
}

export class FunctionApplication extends Expression {
  kind = "FunctionApplication"
  function: Expression
  argument: Expression

  constructor(func: Expression, arg: Expression, line: number, column: number) {
    super(line, column)
    this.function = func
    this.argument = arg
  }
}

export class Pipeline extends Expression {
  kind = "Pipeline"
  left: Expression
  right: Expression

  constructor(
    left: Expression,
    right: Expression,
    line: number,
    column: number
  ) {
    super(line, column)
    this.left = left
    this.right = right
  }
}

export class ReversePipe extends Expression {
  kind = "ReversePipe"
  left: Expression
  right: Expression

  constructor(
    left: Expression,
    right: Expression,
    line: number,
    column: number
  ) {
    super(line, column)
    this.left = left
    this.right = right
  }
}

export class FunctorMap extends Expression {
  kind = "FunctorMap"
  left: Expression
  right: Expression

  constructor(
    left: Expression,
    right: Expression,
    line: number,
    column: number
  ) {
    super(line, column)
    this.left = left
    this.right = right
  }
}

export class ApplicativeApply extends Expression {
  kind = "ApplicativeApply"
  left: Expression
  right: Expression

  constructor(
    left: Expression,
    right: Expression,
    line: number,
    column: number
  ) {
    super(line, column)
    this.left = left
    this.right = right
  }
}

export class MonadBind extends Expression {
  kind = "MonadBind"
  left: Expression
  right: Expression

  constructor(
    left: Expression,
    right: Expression,
    line: number,
    column: number
  ) {
    super(line, column)
    this.left = left
    this.right = right
  }
}

export class FoldMonoid extends Expression {
  kind = "FoldMonoid"
  left: Expression
  right: Expression

  constructor(
    left: Expression,
    right: Expression,
    line: number,
    column: number
  ) {
    super(line, column)
    this.left = left
    this.right = right
  }
}

export class FunctionApplicationOperator extends Expression {
  kind = "FunctionApplicationOperator"
  left: Expression
  right: Expression

  constructor(
    left: Expression,
    right: Expression,
    line: number,
    column: number
  ) {
    super(line, column)
    this.left = left
    this.right = right
  }
}

export class ConstructorExpression extends Expression {
  kind = "ConstructorExpression"
  constructorName: string
  arguments: Expression[]

  constructor(
    constructorName: string,
    args: Expression[],
    line: number,
    column: number
  ) {
    super(line, column)
    this.constructorName = constructorName
    this.arguments = args
  }
}

export class BlockExpression extends Expression {
  kind = "BlockExpression"
  statements: Statement[]
  returnExpression?: Expression

  constructor(
    statements: Statement[],
    returnExpression: Expression | undefined,
    line: number,
    column: number
  ) {
    super(line, column)
    this.statements = statements
    this.returnExpression = returnExpression
  }
}

export class ConditionalExpression extends Expression {
  kind = "ConditionalExpression"
  condition: Expression
  thenExpression: Expression
  elseExpression: Expression

  constructor(
    condition: Expression,
    thenExpr: Expression,
    elseExpr: Expression,
    line: number,
    column: number
  ) {
    super(line, column)
    this.condition = condition
    this.thenExpression = thenExpr
    this.elseExpression = elseExpr
  }
}

// =============================================================================
// Pattern Matching
// =============================================================================

export abstract class Pattern extends ASTNode {}

export class IdentifierPattern extends Pattern {
  kind = "IdentifierPattern"
  name: string

  constructor(name: string, line: number, column: number) {
    super(line, column)
    this.name = name
  }
}

export class LiteralPattern extends Pattern {
  kind = "LiteralPattern"
  value: string | number | boolean

  constructor(value: string | number | boolean, line: number, column: number) {
    super(line, column)
    this.value = value
  }
}

export class ConstructorPattern extends Pattern {
  kind = "ConstructorPattern"
  constructorName: string
  patterns: Pattern[]

  constructor(
    constructorName: string,
    patterns: Pattern[],
    line: number,
    column: number
  ) {
    super(line, column)
    this.constructorName = constructorName
    this.patterns = patterns
  }
}

export class MatchCase extends ASTNode {
  kind = "MatchCase"
  pattern: Pattern
  expression: Expression

  constructor(
    pattern: Pattern,
    expression: Expression,
    line: number,
    column: number
  ) {
    super(line, column)
    this.pattern = pattern
    this.expression = expression
  }
}

export class MatchExpression extends Expression {
  kind = "MatchExpression"
  expression: Expression
  cases: MatchCase[]

  constructor(
    expression: Expression,
    cases: MatchCase[],
    line: number,
    column: number
  ) {
    super(line, column)
    this.expression = expression
    this.cases = cases
  }
}

// =============================================================================
// Statements and Declarations
// =============================================================================

export abstract class Statement extends ASTNode {}

export class ExpressionStatement extends Statement {
  kind = "ExpressionStatement"
  expression: Expression

  constructor(expression: Expression, line: number, column: number) {
    super(line, column)
    this.expression = expression
  }
}

export class Parameter extends ASTNode {
  kind = "Parameter"
  name: string
  type: Type

  constructor(name: string, type: Type, line: number, column: number) {
    super(line, column)
    this.name = name
    this.type = type
  }
}

export class FunctionDeclaration extends Statement {
  kind = "FunctionDeclaration"
  name: string
  parameters: Parameter[]
  returnType: Type
  body: Expression
  isEffectful: boolean

  constructor(
    name: string,
    parameters: Parameter[],
    returnType: Type,
    body: Expression,
    isEffectful: boolean,
    line: number,
    column: number
  ) {
    super(line, column)
    this.name = name
    this.parameters = parameters
    this.returnType = returnType
    this.body = body
    this.isEffectful = isEffectful
  }
}

export class VariableDeclaration extends Statement {
  kind = "VariableDeclaration"
  name: string
  type?: Type
  initializer: Expression

  constructor(
    name: string,
    initializer: Expression,
    type: Type | undefined,
    line: number,
    column: number
  ) {
    super(line, column)
    this.name = name
    this.type = type
    this.initializer = initializer
  }
}

export class TypeField extends ASTNode {
  kind = "TypeField"
  name: string
  type: Type

  constructor(name: string, type: Type, line: number, column: number) {
    super(line, column)
    this.name = name
    this.type = type
  }
}

export class TypeDeclaration extends Statement {
  kind = "TypeDeclaration"
  name: string
  fields: TypeField[]

  constructor(name: string, fields: TypeField[], line: number, column: number) {
    super(line, column)
    this.name = name
    this.fields = fields
  }
}

export class MethodDeclaration extends ASTNode {
  kind = "MethodDeclaration"
  name: string
  parameters: Parameter[]
  returnType: Type
  body: Expression

  constructor(
    name: string,
    parameters: Parameter[],
    returnType: Type,
    body: Expression,
    line: number,
    column: number
  ) {
    super(line, column)
    this.name = name
    this.parameters = parameters
    this.returnType = returnType
    this.body = body
  }
}

export class OperatorDeclaration extends ASTNode {
  kind = "OperatorDeclaration"
  operator: string
  parameters: Parameter[]
  returnType: Type
  body: Expression

  constructor(
    operator: string,
    parameters: Parameter[],
    returnType: Type,
    body: Expression,
    line: number,
    column: number
  ) {
    super(line, column)
    this.operator = operator
    this.parameters = parameters
    this.returnType = returnType
    this.body = body
  }
}

export class MonoidDeclaration extends ASTNode {
  kind = "MonoidDeclaration"
  identity: Expression
  operator: OperatorDeclaration

  constructor(
    identity: Expression,
    operator: OperatorDeclaration,
    line: number,
    column: number
  ) {
    super(line, column)
    this.identity = identity
    this.operator = operator
  }
}

export class ImplBlock extends Statement {
  kind = "ImplBlock"
  typeName: string
  methods: MethodDeclaration[]
  operators: OperatorDeclaration[]
  monoid?: MonoidDeclaration

  constructor(
    typeName: string,
    methods: MethodDeclaration[],
    operators: OperatorDeclaration[],
    monoid: MonoidDeclaration | undefined,
    line: number,
    column: number
  ) {
    super(line, column)
    this.typeName = typeName
    this.methods = methods
    this.operators = operators
    this.monoid = monoid
  }
}

export class ImportItem extends ASTNode {
  kind = "ImportItem"
  name: string
  alias?: string

  constructor(
    name: string,
    alias: string | undefined,
    line: number,
    column: number
  ) {
    super(line, column)
    this.name = name
    this.alias = alias
  }
}

export class ImportDeclaration extends Statement {
  kind = "ImportDeclaration"
  module: string
  items: ImportItem[]

  constructor(
    module: string,
    items: ImportItem[],
    line: number,
    column: number
  ) {
    super(line, column)
    this.module = module
    this.items = items
  }
}

export class ReturnStatement extends Statement {
  kind = "ReturnStatement"
  expression: Expression

  constructor(expression: Expression, line: number, column: number) {
    super(line, column)
    this.expression = expression
  }
}

// =============================================================================
// Program Root
// =============================================================================

export class Program extends ASTNode {
  kind = "Program"
  statements: Statement[]

  constructor(statements: Statement[], line: number = 1, column: number = 1) {
    super(line, column)
    this.statements = statements
  }
}
