/**
 * Seseragi Parser - Recursive Descent Parser
 */

import { type Token, TokenType, Lexer } from "./lexer"
import * as AST from "./ast"
import { TypeVariable } from "./type-inference"

export class ParseError extends Error {
  token: Token

  constructor(message: string, token: Token) {
    super(`${message} at line ${token.line}, column ${token.column}`)
    this.token = token
  }
}

export interface ParseResult {
  statements?: AST.Statement[]
  errors: ParseError[]
}

export class Parser {
  private tokens: Token[]
  private current: number = 0
  private typeVarCounter: number = 0
  private methodRegistry: Map<string, Set<string>> = new Map() // typeName -> method names
  private variableTypes: Map<string, string> = new Map() // variable name -> type name

  constructor(input: string | Token[]) {
    if (typeof input === "string") {
      const lexer = new Lexer(input)
      this.tokens = lexer.tokenize()
    } else {
      this.tokens = input
    }
  }

  private freshTypeVariable(line: number, column: number): TypeVariable {
    return new TypeVariable(this.typeVarCounter++, line, column)
  }

  parse(): ParseResult {
    const statements: AST.Statement[] = []
    const errors: ParseError[] = []

    while (!this.isAtEnd()) {
      if (
        this.peek().type === TokenType.NEWLINE ||
        this.peek().type === TokenType.COMMENT
      ) {
        this.advance()
        continue
      }

      try {
        const stmt = this.statement()
        if (stmt) {
          statements.push(stmt)
        }
      } catch (error) {
        if (error instanceof ParseError) {
          errors.push(error)
          // Try to recover by advancing to the next statement
          this.synchronize()
        } else {
          throw error
        }
      }
    }

    return { statements, errors }
  }

  private synchronize(): void {
    this.advance()

    while (!this.isAtEnd()) {
      if (this.previous().type === TokenType.NEWLINE) {
        return
      }

      switch (this.peek().type) {
        case TokenType.FN:
        case TokenType.LET:
        case TokenType.TYPE:
        case TokenType.STRUCT:
        case TokenType.IMPL:
        case TokenType.IMPORT:
          return
      }

      this.advance()
    }
  }

  // =============================================================================
  // Statements
  // =============================================================================

  private statement(): AST.Statement | null {
    try {
      if (this.match(TokenType.FN)) {
        return this.functionDeclaration(false)
      }

      if (this.match(TokenType.EFFECTFUL)) {
        this.consume(TokenType.FN, "Expected 'fn' after 'effectful'")
        return this.functionDeclaration(true)
      }

      if (this.match(TokenType.TYPE)) {
        return this.typeDeclaration()
      }

      if (this.match(TokenType.STRUCT)) {
        return this.structDeclaration()
      }

      if (this.match(TokenType.IMPL)) {
        return this.implBlock()
      }

      if (this.match(TokenType.IMPORT)) {
        return this.importDeclaration()
      }

      if (this.match(TokenType.LET)) {
        return this.variableDeclaration()
      }

      if (this.match(TokenType.RETURN)) {
        return this.returnStatement()
      }

      // Skip whitespace, newlines, and comments
      if (
        this.peek().type === TokenType.NEWLINE ||
        this.peek().type === TokenType.WHITESPACE ||
        this.peek().type === TokenType.COMMENT
      ) {
        this.advance()
        return null
      }

      // 式ステートメントとして処理
      const expr = this.expression()
      return new AST.ExpressionStatement(
        expr,
        this.previous().line,
        this.previous().column
      )
    } catch (error) {
      this.synchronize()
      throw error
    }
  }

  private functionDeclaration(isEffectful: boolean): AST.FunctionDeclaration {
    const name = this.consume(
      TokenType.IDENTIFIER,
      "Expected function name"
    ).value

    const parameters: AST.Parameter[] = []
    const returnType = this.parseFunctionSignature(parameters)

    let body: AST.Expression

    if (this.match(TokenType.ASSIGN)) {
      // ワンライナー形式: fn name params = expression
      this.skipNewlines()
      body = this.expression()
    } else if (this.match(TokenType.LEFT_BRACE)) {
      // ブロック形式: fn name params { statements }
      body = this.blockExpression()
    } else {
      throw new ParseError(
        "Expected '=' or '{' after function signature",
        this.peek()
      )
    }

    return new AST.FunctionDeclaration(
      name,
      parameters,
      returnType,
      body,
      isEffectful,
      this.previous().line,
      this.previous().column
    )
  }

  private parseFunctionSignature(
    parameters: AST.Parameter[],
    implContext?: string
  ): AST.Type {
    // Check for immediate arrow (no parameters case: "fn name -> Type")
    if (this.check(TokenType.ARROW)) {
      this.advance() // consume ->
      return this.parseType()
    }

    let hasTypedParameters = false

    // Parse parameters until we reach assignment or return type
    while (this.check(TokenType.IDENTIFIER)) {
      const paramNameToken = this.peek()

      // Check if this identifier is followed by colon (typed parameter)
      if (this.checkNext(TokenType.COLON)) {
        // Typed parameter: param :Type ->
        hasTypedParameters = true
        const paramName = this.advance().value
        this.consume(TokenType.COLON, "Expected ':'")
        const paramType = this.parseType()
        this.consume(TokenType.ARROW, "Expected '->' after parameter type")

        // 型ありパラメータでも暗黙的selfとotherをチェック
        const isImplicitSelf =
          implContext && paramName === "self" && parameters.length === 0
        const isImplicitOther = implContext && paramName === "other"

        parameters.push(
          new AST.Parameter(
            paramName,
            paramType,
            paramNameToken.line,
            paramNameToken.column,
            isImplicitSelf,
            isImplicitOther
          )
        )
      } else {
        // Either untyped parameter or return type
        // Look ahead to determine which one
        if (this.isReturnType(hasTypedParameters)) {
          // This is the return type, stop parsing parameters
          return this.parseType()
        } else {
          // Untyped parameter
          const paramName = this.advance().value

          // 暗黙的selfの検出：implコンテキスト内で最初のパラメータが'self'の場合
          const isImplicitSelf =
            implContext && paramName === "self" && parameters.length === 0

          // 暗黙的otherの検出：implコンテキスト内で'other'パラメータの場合
          const isImplicitOther = implContext && paramName === "other"

          if (isImplicitSelf) {
            // 暗黙的selfパラメータ：型は後でimplコンテキストから解決される
            parameters.push(
              new AST.Parameter(
                paramName,
                this.freshTypeVariable(
                  paramNameToken.line,
                  paramNameToken.column
                ),
                paramNameToken.line,
                paramNameToken.column,
                true, // isImplicitSelf = true
                false // isImplicitOther = false
              )
            )
          } else if (isImplicitOther) {
            // 暗黙的otherパラメータ：型は後でimplコンテキストから解決される
            parameters.push(
              new AST.Parameter(
                paramName,
                this.freshTypeVariable(
                  paramNameToken.line,
                  paramNameToken.column
                ),
                paramNameToken.line,
                paramNameToken.column,
                false, // isImplicitSelf = false
                true // isImplicitOther = true
              )
            )
          } else {
            // 通常の型なしパラメータ
            parameters.push(
              new AST.Parameter(
                paramName,
                this.freshTypeVariable(
                  paramNameToken.line,
                  paramNameToken.column
                ),
                paramNameToken.line,
                paramNameToken.column,
                false, // isImplicitSelf = false
                false // isImplicitOther = false
              )
            )
          }

          // 型なしパラメータの後に'->'がある場合は消費（次のパラメータまたは戻り値型）
          if (this.check(TokenType.ARROW)) {
            this.advance() // consume ->
          }
        }
      }
    }

    // パラメータ解析ループが終了した後、明示的な戻り値型があるかチェック
    if (this.check(TokenType.ARROW)) {
      this.advance() // consume ->
      return this.parseType()
    }

    // If we reach here and have no explicit return type, use fresh type variable for inference
    return this.freshTypeVariable(this.peek().line, this.peek().column)
  }

  private checkNext(type: TokenType): boolean {
    if (this.current + 1 >= this.tokens.length) return false
    return this.tokens[this.current + 1].type === type
  }

  private isReturnType(hasTypedParameters: boolean): boolean {
    // Look ahead to see what comes after this identifier
    let lookahead = this.current + 1
    let genericDepth = 0

    // Skip the current identifier and any generic type arguments
    while (lookahead < this.tokens.length) {
      const tokenType = this.tokens[lookahead].type

      // Track generic type depth
      if (tokenType === TokenType.LESS_THAN) {
        genericDepth++
        lookahead++
        continue
      }

      if (tokenType === TokenType.GREATER_THAN) {
        genericDepth--
        lookahead++
        continue
      }

      // If we're inside a generic type, skip all tokens including identifiers
      if (genericDepth > 0) {
        lookahead++
        continue
      }

      // If we find '=' or '{' after this identifier:
      // - In typed functions (e.g., "fn add a :Int -> b :Int -> Int = ..."),
      //   this identifier is a return type
      // - In untyped functions (e.g., "fn add x y = ..."),
      //   this identifier is also a return type if preceded by ->
      if (
        tokenType === TokenType.ASSIGN ||
        tokenType === TokenType.LEFT_BRACE
      ) {
        // Check if this identifier was preceded by -> (indicating return type)
        if (
          this.current > 0 &&
          this.tokens[this.current - 1].type === TokenType.ARROW
        ) {
          return true // This is a return type
        }
        return hasTypedParameters
      }

      // If we find '->' after this identifier, this is NOT a return type
      // (it's a parameter type, and the return type comes later)
      if (tokenType === TokenType.ARROW) {
        return false
      }

      // If we find another identifier outside of generics without encountering anything definitive,
      // this is likely a parameter name, not a return type
      if (tokenType === TokenType.IDENTIFIER) {
        return false
      }

      lookahead++
    }

    return false
  }

  private typeDeclaration(): AST.TypeDeclaration | AST.TypeAliasDeclaration {
    const name = this.consume(TokenType.IDENTIFIER, "Expected type name").value

    // Check if this is a union type (type Name = A | B | C), type alias (type Name = Type), or struct type (type Name { field: Type })
    if (this.match(TokenType.ASSIGN)) {
      this.skipNewlines() // Allow newlines after '='

      // Look ahead to determine if this is a type alias or union type
      // Save current position to backtrack if needed
      const savedPosition = this.current

      // Try to parse the first part and look for a pipe separator
      let isTypeAlias = true
      try {
        // Skip leading newlines/whitespace
        this.skipNewlines()

        // Check for function type parentheses
        if (this.check(TokenType.LEFT_PAREN)) {
          // Function type like (Int -> Bool) - always a type alias
          isTypeAlias = true
        } else if (this.check(TokenType.IDENTIFIER)) {
          this.advance() // consume the identifier

          // Parse any type arguments (e.g., Maybe<String>)
          if (this.match(TokenType.LESS_THAN)) {
            // Skip the generic arguments to look for pipes later
            let bracketCount = 1
            while (bracketCount > 0 && !this.isAtEnd()) {
              if (this.check(TokenType.LESS_THAN)) {
                bracketCount++
              } else if (this.check(TokenType.GREATER_THAN)) {
                bracketCount--
              }
              this.advance()
            }
          }

          // Parse any additional type arguments for constructor variants
          this.parseVariantDataTypes()

          // Skip whitespace and check for pipe
          this.skipNewlines()
          if (this.check(TokenType.PIPE)) {
            isTypeAlias = false // This is a union type
          }
        } else {
          // Other type constructs are type aliases
          isTypeAlias = true
        }
      } catch (e) {
        // If parsing fails, assume it's a complex type alias
        isTypeAlias = true
      }

      // Restore position
      this.current = savedPosition

      if (isTypeAlias) {
        // Type alias: type UserId = Int
        const aliasedType = this.parseType()
        return new AST.TypeAliasDeclaration(
          name,
          aliasedType,
          this.previous().line,
          this.previous().column
        )
      } else {
        // Union type: type Color = Red | Green | Blue
        return this.parseUnionType(name)
      }
    } else {
      // Struct type: type Point { x: Int, y: Int }
      return this.parseStructType(name)
    }
  }

  private parseUnionType(name: string): AST.TypeDeclaration {
    const variants: AST.TypeField[] = []

    // Skip optional newlines and pipes at the beginning
    this.skipNewlines()
    if (this.match(TokenType.PIPE)) {
      this.skipNewlines()
    }

    // Parse first variant
    const firstVariant = this.consume(
      TokenType.IDENTIFIER,
      "Expected variant name"
    ).value

    // Check if variant has associated data types
    const firstVariantTypes = this.parseVariantDataTypes()
    variants.push(
      new AST.TypeField(
        firstVariant,
        firstVariantTypes.length > 0
          ? new AST.GenericType(
              "Tuple",
              firstVariantTypes,
              this.previous().line,
              this.previous().column
            )
          : new AST.PrimitiveType(
              "Unit",
              this.previous().line,
              this.previous().column
            ),
        this.previous().line,
        this.previous().column
      )
    )

    // Parse additional variants separated by '|'
    while (true) {
      this.skipNewlines()
      if (this.match(TokenType.PIPE)) {
        this.skipNewlines()
        const variantName = this.consume(
          TokenType.IDENTIFIER,
          "Expected variant name"
        ).value

        // Check if variant has associated data types
        const variantTypes = this.parseVariantDataTypes()
        variants.push(
          new AST.TypeField(
            variantName,
            variantTypes.length > 0
              ? new AST.GenericType(
                  "Tuple",
                  variantTypes,
                  this.previous().line,
                  this.previous().column
                )
              : new AST.PrimitiveType(
                  "Unit",
                  this.previous().line,
                  this.previous().column
                ),
            this.previous().line,
            this.previous().column
          )
        )
      } else {
        break
      }
    }

    return new AST.TypeDeclaration(
      name,
      variants,
      this.previous().line,
      this.previous().column
    )
  }

  private parseVariantDataTypes(): AST.Type[] {
    const types: AST.Type[] = []

    // Parse type parameters for constructor variants (RGB Int Int Int)
    while (this.check(TokenType.IDENTIFIER) && this.isTypeIdentifier()) {
      types.push(this.parseType())
    }

    return types
  }

  private isTypeIdentifier(): boolean {
    const name = this.peek().value
    // Check if it's a known type name (basic types or capitalized identifiers)
    return (
      name === "Int" ||
      name === "Float" ||
      name === "String" ||
      name === "Bool" ||
      name === "Unit" ||
      name === "Char" ||
      (name[0] === name[0].toUpperCase() &&
        name !== this.tokens[this.current + 1]?.value)
    )
  }

  private structDeclaration(): AST.StructDeclaration {
    const structToken = this.previous()
    const name = this.consume(
      TokenType.IDENTIFIER,
      "Expected struct name"
    ).value

    this.consume(TokenType.LEFT_BRACE, "Expected '{' after struct name")
    this.skipNewlines()

    const fields: AST.RecordField[] = []

    // Parse fields
    while (!this.check(TokenType.RIGHT_BRACE) && !this.isAtEnd()) {
      const fieldName = this.consume(
        TokenType.IDENTIFIER,
        "Expected field name"
      ).value
      this.consume(TokenType.COLON, "Expected ':' after field name")
      const fieldType = this.parseType()

      fields.push(
        new AST.RecordField(
          fieldName,
          fieldType,
          this.previous().line,
          this.previous().column
        )
      )

      // Check for comma or newline
      if (this.match(TokenType.COMMA)) {
        this.skipNewlines()
      } else if (this.check(TokenType.NEWLINE)) {
        this.skipNewlines()
      } else if (!this.check(TokenType.RIGHT_BRACE)) {
        throw new ParseError(
          "Expected ',' or newline after struct field",
          this.peek()
        )
      }
    }

    this.consume(TokenType.RIGHT_BRACE, "Expected '}' after struct fields")

    return new AST.StructDeclaration(
      name,
      fields,
      structToken.line,
      structToken.column
    )
  }

  private parseStructType(name: string): AST.TypeDeclaration {
    this.consume(TokenType.LEFT_BRACE, "Expected '{' after type name")

    const fields: AST.TypeField[] = []

    while (!this.check(TokenType.RIGHT_BRACE) && !this.isAtEnd()) {
      if (this.match(TokenType.NEWLINE)) continue

      const fieldName = this.consume(
        TokenType.IDENTIFIER,
        "Expected field name"
      ).value
      this.consume(TokenType.COLON, "Expected ':' after field name")
      const fieldType = this.parseType()

      fields.push(
        new AST.TypeField(
          fieldName,
          fieldType,
          this.previous().line,
          this.previous().column
        )
      )

      // Handle optional comma and newlines between fields
      if (this.check(TokenType.COMMA)) {
        this.advance()
      }

      // Skip newlines after comma or field
      while (this.match(TokenType.NEWLINE)) {}
    }

    this.consume(TokenType.RIGHT_BRACE, "Expected '}' after type fields")

    return new AST.TypeDeclaration(
      name,
      fields,
      this.previous().line,
      this.previous().column
    )
  }

  private implBlock(): AST.ImplBlock {
    const typeName = this.consume(
      TokenType.IDENTIFIER,
      "Expected type name"
    ).value

    this.consume(TokenType.LEFT_BRACE, "Expected '{' after type name")

    const methods: AST.MethodDeclaration[] = []
    const operators: AST.OperatorDeclaration[] = []
    let monoid: AST.MonoidDeclaration | undefined

    // Initialize method registry for this type early
    if (!this.methodRegistry.has(typeName)) {
      this.methodRegistry.set(typeName, new Set())
    }
    const methodSet = this.methodRegistry.get(typeName)!

    while (!this.check(TokenType.RIGHT_BRACE) && !this.isAtEnd()) {
      if (this.match(TokenType.NEWLINE) || this.match(TokenType.COMMENT))
        continue

      if (this.match(TokenType.FN)) {
        const method = this.methodDeclaration(typeName)
        methods.push(method)
        // Register method immediately for use in subsequent operator declarations
        methodSet.add(method.name)
      } else if (this.match(TokenType.OPERATOR)) {
        operators.push(this.operatorDeclaration(typeName))
      } else if (this.match(TokenType.MONOID)) {
        monoid = this.monoidDeclaration()
      } else {
        throw new ParseError(
          "Expected method, operator, or monoid declaration",
          this.peek()
        )
      }
    }

    this.consume(TokenType.RIGHT_BRACE, "Expected '}' after impl block")

    return new AST.ImplBlock(
      typeName,
      methods,
      operators,
      monoid,
      this.previous().line,
      this.previous().column
    )
  }

  private methodDeclaration(implTypeName?: string): AST.MethodDeclaration {
    const name = this.consume(
      TokenType.IDENTIFIER,
      "Expected method name"
    ).value

    const parameters: AST.Parameter[] = []
    const returnType = this.parseFunctionSignature(parameters, implTypeName)

    let body: AST.Expression

    if (this.match(TokenType.ASSIGN)) {
      // ワンライナー形式: fn name params = expression
      this.skipNewlines()
      body = this.expression()
    } else if (this.match(TokenType.LEFT_BRACE)) {
      // ブロック形式: fn name params { statements }
      body = this.blockExpression()
    } else {
      throw new ParseError(
        "Expected '=' or '{' after method signature",
        this.peek()
      )
    }

    return new AST.MethodDeclaration(
      name,
      parameters,
      returnType,
      body,
      this.previous().line,
      this.previous().column
    )
  }

  private operatorDeclaration(implTypeName?: string): AST.OperatorDeclaration {
    // operator + self: Point -> other: Point -> Point = self add other
    // Check if current token is a valid operator
    const current = this.peek()
    let operator: string

    // Valid operator tokens
    if (
      this.match(
        TokenType.PLUS,
        TokenType.MINUS,
        TokenType.MULTIPLY,
        TokenType.DIVIDE,
        TokenType.MODULO,
        TokenType.EQUAL,
        TokenType.NOT_EQUAL,
        TokenType.LESS_THAN,
        TokenType.GREATER_THAN,
        TokenType.LESS_EQUAL,
        TokenType.GREATER_EQUAL,
        TokenType.AND,
        TokenType.OR
      )
    ) {
      operator = this.previous().value
    } else {
      throw new ParseError(`Invalid operator: ${current.value}`, current)
    }

    const parameters: AST.Parameter[] = []
    const returnType = this.parseFunctionSignature(parameters, implTypeName)

    let body: AST.Expression

    if (this.match(TokenType.ASSIGN)) {
      // ワンライナー形式: operator + params = expression
      this.skipNewlines()
      body = this.expression()
    } else if (this.match(TokenType.LEFT_BRACE)) {
      // ブロック形式: operator + params { statements }
      body = this.blockExpression()
    } else {
      throw new ParseError(
        "Expected '=' or '{' after operator signature",
        this.peek()
      )
    }

    return new AST.OperatorDeclaration(
      operator,
      parameters,
      returnType,
      body,
      this.previous().line,
      this.previous().column
    )
  }

  private monoidDeclaration(): AST.MonoidDeclaration {
    this.consume(TokenType.LEFT_BRACE, "Expected '{' after 'monoid'")

    // Parse identity
    this.consume(TokenType.IDENTIFIER, "Expected 'identity'") // TODO: make this a keyword
    const identity = this.expression()

    // Parse operator
    const operator = this.operatorDeclaration()

    this.consume(TokenType.RIGHT_BRACE, "Expected '}' after monoid")

    return new AST.MonoidDeclaration(
      identity,
      operator,
      this.previous().line,
      this.previous().column
    )
  }

  private importDeclaration(): AST.ImportDeclaration {
    const module = this.consume(
      TokenType.IDENTIFIER,
      "Expected module name"
    ).value

    this.consume(TokenType.COLON, "Expected ':' after module name")
    this.consume(TokenType.COLON, "Expected '::' after module name")
    this.consume(TokenType.LEFT_BRACE, "Expected '{' after '::'")

    const items: AST.ImportItem[] = []

    do {
      const name = this.consume(
        TokenType.IDENTIFIER,
        "Expected import item name"
      ).value
      let alias: string | undefined

      if (this.match(TokenType.AS)) {
        alias = this.consume(TokenType.IDENTIFIER, "Expected alias name").value
      }

      items.push(
        new AST.ImportItem(
          name,
          alias,
          this.previous().line,
          this.previous().column
        )
      )
    } while (this.match(TokenType.COMMA))

    this.consume(TokenType.RIGHT_BRACE, "Expected '}' after import items")

    return new AST.ImportDeclaration(
      module,
      items,
      this.previous().line,
      this.previous().column
    )
  }

  private variableDeclaration(): AST.Statement {
    const line = this.previous().line
    const column = this.previous().column

    // Check if this is tuple destructuring
    if (this.check(TokenType.LEFT_PAREN)) {
      // Tuple destructuring: let (x, y) = tuple
      const pattern = this.pattern() as AST.TuplePattern

      this.consume(TokenType.ASSIGN, "Expected '=' after tuple pattern")
      this.skipNewlines()
      const initializer = this.expression()

      return new AST.TupleDestructuring(pattern, initializer, line, column)
    }
    // Check if this is record destructuring
    else if (this.check(TokenType.LEFT_BRACE)) {
      // Record destructuring: let {x, y} = record
      const pattern = this.recordPattern()

      this.consume(TokenType.ASSIGN, "Expected '=' after record pattern")
      this.skipNewlines()
      const initializer = this.expression()

      return new AST.RecordDestructuring(pattern, initializer, line, column)
    }
    // Check if this is struct destructuring with struct name
    else if (
      this.check(TokenType.IDENTIFIER) &&
      this.checkAhead(TokenType.LEFT_BRACE)
    ) {
      // Struct destructuring: let Person {name, age} = person
      const structName = this.advance().value
      const pattern = this.structPattern(structName)

      this.consume(TokenType.ASSIGN, "Expected '=' after struct pattern")
      this.skipNewlines()
      const initializer = this.expression()

      return new AST.StructDestructuring(pattern, initializer, line, column)
    } else {
      // Regular variable declaration: let x = value
      const name = this.consume(
        TokenType.IDENTIFIER,
        "Expected variable name"
      ).value

      let type: AST.Type | undefined
      if (this.match(TokenType.COLON)) {
        type = this.parseType()
      }

      this.consume(TokenType.ASSIGN, "Expected '=' after variable name")
      this.skipNewlines()
      const initializer = this.expression()

      return new AST.VariableDeclaration(name, initializer, type, line, column)
    }
  }

  private returnStatement(): AST.ReturnStatement {
    const expr = this.expression()

    return new AST.ReturnStatement(
      expr,
      this.previous().line,
      this.previous().column
    )
  }

  // =============================================================================
  // Types
  // =============================================================================

  private parseType(): AST.Type {
    // Check for parenthesized types first (function types, tuple types, or parenthesized types)
    if (this.check(TokenType.LEFT_PAREN)) {
      const token = this.advance() // consume '('
      const line = token.line
      const column = token.column

      const firstType = this.parseType()

      // Check if this is a tuple type (has comma) or function type (has arrow)
      if (this.match(TokenType.COMMA)) {
        // Tuple type (Type1, Type2, Type3)
        const elementTypes = [firstType]

        do {
          elementTypes.push(this.parseType())
        } while (this.match(TokenType.COMMA))

        this.consume(TokenType.RIGHT_PAREN, "Expected ')' after tuple type")

        if (elementTypes.length < 2) {
          throw new ParseError(
            "Tuple types must have at least 2 elements",
            this.previous()
          )
        }

        return new AST.TupleType(elementTypes, line, column)
      } else if (this.match(TokenType.ARROW)) {
        // Function type (Param -> Return)
        const returnType = this.parseType()
        this.consume(TokenType.RIGHT_PAREN, "Expected ')' after function type")

        return new AST.FunctionType(firstType, returnType, line, column)
      } else {
        // Single type in parentheses
        this.consume(TokenType.RIGHT_PAREN, "Expected ')' after type")
        return firstType
      }
    }

    const token = this.advance()

    if (token.type === TokenType.IDENTIFIER) {
      const name = token.value

      // Check for generic types List<T>, Maybe<T>, etc.
      if (this.match(TokenType.LESS_THAN)) {
        const typeArgs: AST.Type[] = []

        do {
          typeArgs.push(this.parseType())
        } while (this.match(TokenType.COMMA))

        this.consume(
          TokenType.GREATER_THAN,
          "Expected '>' after type arguments"
        )

        return new AST.GenericType(name, typeArgs, token.line, token.column)
      }

      return new AST.PrimitiveType(name, token.line, token.column)
    }

    if (token.type === TokenType.LEFT_BRACE) {
      // Record type { name: Type, age: Int }
      const fields: AST.RecordField[] = []

      if (!this.check(TokenType.RIGHT_BRACE)) {
        do {
          this.skipNewlines()
          const fieldName = this.consume(
            TokenType.IDENTIFIER,
            "Expected field name"
          ).value
          this.skipNewlines()
          this.consume(TokenType.COLON, "Expected ':' after field name")
          this.skipNewlines()
          const fieldType = this.parseType()

          fields.push(
            new AST.RecordField(
              fieldName,
              fieldType,
              this.previous().line,
              this.previous().column
            )
          )
          this.skipNewlines()
        } while (this.match(TokenType.COMMA))
      }

      this.skipNewlines()

      this.consume(TokenType.RIGHT_BRACE, "Expected '}' after record fields")

      return new AST.RecordType(fields, token.line, token.column)
    }

    throw new ParseError("Expected type", token)
  }

  // =============================================================================
  // Expressions
  // =============================================================================

  private expression(): AST.Expression {
    return this.conditionalExpression()
  }

  // パイプライン演算子を処理しない式（配列リテラルの最初の要素用）
  private expressionWithoutPipeline(): AST.Expression {
    return this.conditionalExpressionWithoutPipeline()
  }

  private conditionalExpression(): AST.Expression {
    if (this.match(TokenType.IF)) {
      const condition = this.binaryExpression()
      this.skipNewlines()
      this.consume(TokenType.THEN, "Expected 'then' after condition")
      this.skipNewlines()
      const thenExpr = this.conditionalExpression()
      this.skipNewlines()
      this.consume(TokenType.ELSE, "Expected 'else' after then expression")
      this.skipNewlines()
      const elseExpr = this.conditionalExpression()

      return new AST.ConditionalExpression(
        condition,
        thenExpr,
        elseExpr,
        this.previous().line,
        this.previous().column
      )
    }

    if (this.match(TokenType.MATCH)) {
      return this.matchExpression()
    }

    return this.binaryExpression()
  }

  private conditionalExpressionWithoutPipeline(): AST.Expression {
    if (this.match(TokenType.IF)) {
      const condition = this.binaryExpressionWithoutPipeline()
      this.skipNewlines()
      this.consume(TokenType.THEN, "Expected 'then' after condition")
      this.skipNewlines()
      const thenExpr = this.conditionalExpressionWithoutPipeline()
      this.skipNewlines()
      this.consume(TokenType.ELSE, "Expected 'else' after then expression")
      this.skipNewlines()
      const elseExpr = this.conditionalExpressionWithoutPipeline()

      return new AST.ConditionalExpression(
        condition,
        thenExpr,
        elseExpr,
        this.previous().line,
        this.previous().column
      )
    }

    if (this.match(TokenType.MATCH)) {
      return this.matchExpression()
    }

    return this.binaryExpressionWithoutPipeline()
  }

  private matchExpression(): AST.MatchExpression {
    const expr = this.primaryExpression()

    this.consume(TokenType.LEFT_BRACE, "Expected '{' after match expression")

    const cases: AST.MatchCase[] = []

    while (!this.check(TokenType.RIGHT_BRACE) && !this.isAtEnd()) {
      if (this.match(TokenType.NEWLINE)) continue

      const pattern = this.pattern()
      this.consume(TokenType.ARROW, "Expected '->' after pattern")
      const caseExpr = this.expression()

      cases.push(
        new AST.MatchCase(
          pattern,
          caseExpr,
          this.previous().line,
          this.previous().column
        )
      )
    }

    this.consume(TokenType.RIGHT_BRACE, "Expected '}' after match cases")

    return new AST.MatchExpression(
      expr,
      cases,
      this.previous().line,
      this.previous().column
    )
  }

  private ternaryExpression(): AST.Expression {
    let expr = this.functionApplicationExpression()

    if (this.match(TokenType.QUESTION)) {
      const startLine = this.previous().line
      const startColumn = this.previous().column

      // cons演算子（:）との競合を避けるため、rangeExpressionレベルで解析
      // cons演算子を使用する場合は括弧が必要: condition ? (head : tail) : []
      const trueExpr = this.rangeExpression()
      this.consume(
        TokenType.COLON,
        "Expected ':' after true expression in ternary operator"
      )
      this.skipNewlines() // コロンの後の改行をスキップ
      const falseExpr = this.ternaryExpression()

      expr = new AST.TernaryExpression(
        expr,
        trueExpr,
        falseExpr,
        startLine,
        startColumn
      )
    }

    return expr
  }

  private ternaryExpressionWithoutPipeline(): AST.Expression {
    let expr = this.functionApplicationExpressionWithoutPipeline()

    if (this.match(TokenType.QUESTION)) {
      const startLine = this.previous().line
      const startColumn = this.previous().column

      // cons演算子（:）との競合を避けるため、rangeExpressionレベルで解析
      // cons演算子を使用する場合は括弧が必要: condition ? (head : tail) : []
      const trueExpr = this.rangeExpression()
      this.consume(
        TokenType.COLON,
        "Expected ':' after true expression in ternary operator"
      )
      this.skipNewlines() // コロンの後の改行をスキップ
      const falseExpr = this.ternaryExpressionWithoutPipeline()

      expr = new AST.TernaryExpression(
        expr,
        trueExpr,
        falseExpr,
        startLine,
        startColumn
      )
    }

    return expr
  }

  private pattern(): AST.Pattern {
    // Wildcard pattern
    if (this.match(TokenType.WILDCARD)) {
      return new AST.WildcardPattern(
        this.previous().line,
        this.previous().column
      )
    }

    // Tuple pattern
    if (this.match(TokenType.LEFT_PAREN)) {
      const line = this.previous().line
      const column = this.previous().column

      const patterns: AST.Pattern[] = []

      // Parse first pattern
      patterns.push(this.pattern())

      // Must have comma for tuple pattern
      if (!this.match(TokenType.COMMA)) {
        throw new ParseError("Expected ',' in tuple pattern", this.peek())
      }

      // Parse remaining patterns
      do {
        patterns.push(this.pattern())
      } while (this.match(TokenType.COMMA))

      this.consume(TokenType.RIGHT_PAREN, "Expected ')' after tuple pattern")

      if (patterns.length < 2) {
        throw new ParseError(
          "Tuple patterns must have at least 2 elements",
          this.previous()
        )
      }

      return new AST.TuplePattern(patterns, line, column)
    }

    if (this.check(TokenType.IDENTIFIER)) {
      const name = this.advance().value

      // Check if this is a constructor pattern by looking ahead
      // A constructor pattern has either more identifiers or is followed by ->
      const isConstructor = this.isConstructorName(name)

      if (isConstructor && this.check(TokenType.IDENTIFIER)) {
        // Constructor pattern with arguments
        const patterns: AST.Pattern[] = []

        while (
          this.check(TokenType.IDENTIFIER) &&
          !this.checkAhead(TokenType.ARROW)
        ) {
          patterns.push(this.pattern())
        }

        return new AST.ConstructorPattern(
          name,
          patterns,
          this.previous().line,
          this.previous().column
        )
      } else if (isConstructor && !this.check(TokenType.IDENTIFIER)) {
        // Constructor pattern without arguments (Red, Green, Blue)
        return new AST.ConstructorPattern(
          name,
          [],
          this.previous().line,
          this.previous().column
        )
      }

      // Variable pattern
      return new AST.IdentifierPattern(
        name,
        this.previous().line,
        this.previous().column
      )
    }

    if (
      this.check(TokenType.INTEGER) ||
      this.check(TokenType.FLOAT) ||
      this.check(TokenType.STRING) ||
      this.check(TokenType.BOOLEAN)
    ) {
      const token = this.advance()
      let value: string | number | boolean = token.value

      if (token.type === TokenType.INTEGER) {
        value = parseInt(token.value)
      } else if (token.type === TokenType.FLOAT) {
        value = parseFloat(token.value)
      } else if (token.type === TokenType.BOOLEAN) {
        value = token.value === "True"
      }

      return new AST.LiteralPattern(value, token.line, token.column)
    }

    throw new ParseError("Expected pattern", this.peek())
  }

  private binaryExpression(): AST.Expression {
    return this.ternaryExpression()
  }

  private binaryExpressionWithoutPipeline(): AST.Expression {
    return this.ternaryExpressionWithoutPipeline()
  }

  private functionApplicationExpression(): AST.Expression {
    let expr = this.pipelineExpression()

    while (true) {
      this.skipNewlines()
      if (this.match(TokenType.FUNCTION_APPLICATION)) {
        this.skipNewlines()
        const right = this.functionApplicationExpression() // 右結合のため再帰
        expr = new AST.FunctionApplicationOperator(
          expr,
          right,
          this.previous().line,
          this.previous().column
        )
      } else {
        break
      }
    }

    return expr
  }

  private functionApplicationExpressionWithoutPipeline(): AST.Expression {
    let expr = this.bindExpression() // パイプライン演算子をスキップ

    while (true) {
      this.skipNewlines()
      if (this.match(TokenType.FUNCTION_APPLICATION)) {
        this.skipNewlines()
        const right = this.functionApplicationExpressionWithoutPipeline() // 右結合のため再帰
        expr = new AST.FunctionApplicationOperator(
          expr,
          right,
          this.previous().line,
          this.previous().column
        )
      } else {
        break
      }
    }

    return expr
  }

  private pipelineExpression(): AST.Expression {
    let expr = this.bindExpression()

    while (true) {
      this.skipNewlines()
      if (this.match(TokenType.PIPE)) {
        this.skipNewlines()
        const right = this.bindExpression()
        expr = new AST.Pipeline(
          expr,
          right,
          this.previous().line,
          this.previous().column
        )
      } else {
        break
      }
    }

    return expr
  }

  private bindExpression(): AST.Expression {
    let expr = this.applicativeExpression()

    while (true) {
      this.skipNewlines()
      if (this.match(TokenType.BIND)) {
        this.skipNewlines()
        const right = this.applicativeExpression()
        expr = new AST.MonadBind(
          expr,
          right,
          this.previous().line,
          this.previous().column
        )
      } else {
        break
      }
    }

    return expr
  }

  private applicativeExpression(): AST.Expression {
    let expr = this.functorExpression()

    while (true) {
      this.skipNewlines()
      if (this.match(TokenType.APPLY)) {
        this.skipNewlines()
        const right = this.functorExpression()
        expr = new AST.ApplicativeApply(
          expr,
          right,
          this.previous().line,
          this.previous().column
        )
      } else {
        break
      }
    }

    return expr
  }

  private functorExpression(): AST.Expression {
    let expr = this.reversePipeExpression()

    while (true) {
      this.skipNewlines()
      if (this.match(TokenType.MAP)) {
        this.skipNewlines()
        const right = this.reversePipeExpression()
        expr = new AST.FunctorMap(
          expr,
          right,
          this.previous().line,
          this.previous().column
        )
      } else {
        break
      }
    }

    return expr
  }

  private reversePipeExpression(): AST.Expression {
    let expr = this.consExpression()

    while (true) {
      this.skipNewlines()
      if (this.match(TokenType.REVERSE_PIPE)) {
        this.skipNewlines()
        const right = this.consExpression()
        expr = new AST.ReversePipe(
          expr,
          right,
          this.previous().line,
          this.previous().column
        )
      } else {
        break
      }
    }

    return expr
  }

  private consExpression(): AST.Expression {
    let expr = this.rangeExpression()

    // Right associative CONS operator (:)
    if (this.match(TokenType.COLON)) {
      this.skipNewlines()
      const right = this.consExpression() // Right associative - recurse to same level
      expr = new AST.BinaryOperation(
        expr,
        ":",
        right,
        this.previous().line,
        this.previous().column
      )
    }

    return expr
  }

  private rangeExpression(): AST.Expression {
    let expr = this.logicalOrExpression()

    if (this.match(TokenType.RANGE, TokenType.RANGE_INCLUSIVE)) {
      const inclusive = this.previous().type === TokenType.RANGE_INCLUSIVE
      const end = this.logicalOrExpression()
      expr = new AST.RangeLiteral(
        expr,
        end,
        inclusive,
        this.previous().line,
        this.previous().column
      )
    }

    return expr
  }

  private logicalOrExpression(): AST.Expression {
    let expr = this.logicalAndExpression()

    while (this.match(TokenType.OR)) {
      const operator = this.previous().value
      const right = this.logicalAndExpression()
      expr = new AST.BinaryOperation(
        expr,
        operator,
        right,
        this.previous().line,
        this.previous().column
      )
    }

    return expr
  }

  private logicalAndExpression(): AST.Expression {
    let expr = this.comparisonExpression()

    while (this.match(TokenType.AND)) {
      const operator = this.previous().value
      const right = this.comparisonExpression()
      expr = new AST.BinaryOperation(
        expr,
        operator,
        right,
        this.previous().line,
        this.previous().column
      )
    }

    return expr
  }

  private comparisonExpression(): AST.Expression {
    let expr = this.termExpression()

    while (
      this.match(
        TokenType.EQUAL,
        TokenType.NOT_EQUAL,
        TokenType.GREATER_THAN,
        TokenType.GREATER_EQUAL,
        TokenType.LESS_THAN,
        TokenType.LESS_EQUAL
      )
    ) {
      const operator = this.previous().value
      const right = this.termExpression()
      expr = new AST.BinaryOperation(
        expr,
        operator,
        right,
        this.previous().line,
        this.previous().column
      )
    }

    return expr
  }

  private termExpression(): AST.Expression {
    let expr = this.factorExpression()

    while (this.match(TokenType.PLUS, TokenType.MINUS)) {
      const operator = this.previous().value
      const right = this.factorExpression()
      expr = new AST.BinaryOperation(
        expr,
        operator,
        right,
        this.previous().line,
        this.previous().column
      )
    }

    return expr
  }

  private factorExpression(): AST.Expression {
    let expr = this.unaryExpression()

    while (this.match(TokenType.MULTIPLY, TokenType.DIVIDE, TokenType.MODULO)) {
      const operator = this.previous().value
      const right = this.unaryExpression()
      expr = new AST.BinaryOperation(
        expr,
        operator,
        right,
        this.previous().line,
        this.previous().column
      )
    }

    return expr
  }

  private unaryExpression(): AST.Expression {
    if (this.match(TokenType.NOT, TokenType.MINUS)) {
      const operator = this.previous().value
      const expr = this.unaryExpression()
      return new AST.UnaryOperation(
        operator,
        expr,
        this.previous().line,
        this.previous().column
      )
    }

    // Handle prefix operators ^ and >> as function applications
    if (this.match(TokenType.HEAD_OP, TokenType.TAIL_OP)) {
      const operator = this.previous()
      const line = operator.line
      const column = operator.column
      const functionName = operator.type === TokenType.HEAD_OP ? "head" : "tail"
      
      // Check for tail chaining pattern: >> . >> list
      if (operator.type === TokenType.TAIL_OP && this.check(TokenType.DOT)) {
        // Start building the chain
        let expr = new AST.FunctionApplication(
          new AST.Identifier("tail", line, column),
          this.parseTailChain(),
          line,
          column
        )
        return expr
      }
      
      // Parse the argument normally
      const arg = this.unaryExpression()
      
      // Return as a FunctionApplication
      return new AST.FunctionApplication(
        new AST.Identifier(functionName, line, column),
        arg,
        line,
        column
      )
    }


    return this.callExpression()
  }

  // 関数適用の引数として使用する単項演算子専用のパーサー
  private parseUnaryOnly(): AST.Expression {
    if (this.match(TokenType.NOT, TokenType.MINUS)) {
      const operator = this.previous().value
      const expr = this.parseUnaryOnly()
      return new AST.UnaryOperation(
        operator,
        expr,
        this.previous().line,
        this.previous().column
      )
    }

    // Handle prefix operators ^ and >> as function applications
    if (this.match(TokenType.HEAD_OP, TokenType.TAIL_OP)) {
      const operator = this.previous()
      const line = operator.line
      const column = operator.column
      const functionName = operator.type === TokenType.HEAD_OP ? "head" : "tail"
      
      // Check for tail chaining pattern: >> . >> list
      if (operator.type === TokenType.TAIL_OP && this.check(TokenType.DOT)) {
        // Start building the chain
        let expr = new AST.FunctionApplication(
          new AST.Identifier("tail", line, column),
          this.parseTailChain(),
          line,
          column
        )
        return expr
      }
      
      // Parse the argument normally
      const arg = this.parseUnaryOnly()
      
      // Return as a FunctionApplication
      return new AST.FunctionApplication(
        new AST.Identifier(functionName, line, column),
        arg,
        line,
        column
      )
    }


    return this.primaryExpression()
  }

  // Parse tail chaining pattern: . >> . >> list
  private parseTailChain(): AST.Expression {
    // We expect: . >> [. >> ...] list
    
    // Consume the first dot
    this.consume(TokenType.DOT, "Expected '.' in tail chain")
    
    // Check if we have another >> 
    if (this.match(TokenType.TAIL_OP)) {
      const line = this.previous().line
      const column = this.previous().column
      
      // Check if there's another dot (continuing the chain)
      if (this.check(TokenType.DOT)) {
        // Recursive chain: tail(parseTailChain())
        return new AST.FunctionApplication(
          new AST.Identifier("tail", line, column),
          this.parseTailChain(),
          line,
          column
        )
      } else {
        // End of chain: tail(expression)
        return new AST.FunctionApplication(
          new AST.Identifier("tail", line, column),
          this.unaryExpression(),
          line,
          column
        )
      }
    } else {
      throw new Error(`Expected '>>' after '.' in tail chain at line ${this.peek().line}`)
    }
  }

  private callExpression(): AST.Expression {
    let expr = this.primaryExpression()

    while (true) {
      if (this.match(TokenType.DOT)) {
        // Record field access: obj.field
        const fieldName = this.consume(
          TokenType.IDENTIFIER,
          "Expected field name after '.'"
        ).value
        expr = new AST.RecordAccess(
          expr,
          fieldName,
          this.previous().line,
          this.previous().column
        )
      } else if (this.match(TokenType.LEFT_BRACKET)) {
        // Array access: array[index]
        const index = this.expression()
        this.consume(TokenType.RIGHT_BRACKET, "Expected ']' after array index")
        expr = new AST.ArrayAccess(
          expr,
          index,
          this.previous().line,
          this.previous().column
        )
      } else if (this.match(TokenType.LEFT_PAREN)) {
        // 括弧付き関数呼び出し
        const args: AST.Expression[] = []

        if (!this.check(TokenType.RIGHT_PAREN)) {
          do {
            args.push(this.expression())
          } while (this.match(TokenType.COMMA))
        }

        this.consume(TokenType.RIGHT_PAREN, "Expected ')' after arguments")
        expr = new AST.FunctionCall(
          expr,
          args,
          this.previous().line,
          this.previous().column
        )
      } else if (
        this.check(TokenType.IDENTIFIER) &&
        this.isMethodCall() &&
        this.isActualMethodCall(expr)
      ) {
        // メソッド呼び出し構文: obj methodName arg1 arg2
        // Enhanced method call detection to avoid conflicts with function application
        const methodName = this.advance().value
        const args: AST.Expression[] = []

        // 引数なしの括弧付きメソッド呼び出しをチェック: obj method()
        if (this.check(TokenType.LEFT_PAREN)) {
          this.advance() // consume '('

          if (!this.check(TokenType.RIGHT_PAREN)) {
            // 括弧内に引数がある場合
            do {
              args.push(this.expression())
            } while (this.match(TokenType.COMMA))
          }

          this.consume(
            TokenType.RIGHT_PAREN,
            "Expected ')' after method arguments"
          )
        } else {
          // メソッドの引数を収集（スペース区切り）
          while (
            this.canStartExpression() &&
            !this.check(TokenType.NEWLINE) &&
            !this.isAtEnd()
          ) {
            const arg = this.check(TokenType.NOT)
              ? this.parseUnaryOnly()
              : this.primaryExpression()
            args.push(arg)

            // 式の終了条件をチェック
            if (
              this.check(TokenType.NEWLINE) ||
              this.check(TokenType.SEMICOLON) ||
              this.check(TokenType.RIGHT_PAREN) ||
              this.check(TokenType.RIGHT_BRACE) ||
              this.check(TokenType.COMMA) ||
              this.check(TokenType.PIPE) ||
              this.check(TokenType.REVERSE_PIPE) ||
              this.check(TokenType.BIND)
            ) {
              break
            }
          }
        }

        expr = new AST.MethodCall(
          expr,
          methodName,
          args,
          this.previous().line,
          this.previous().column
        )
      } else if (this.canStartExpression()) {
        // 括弧なし関数適用（関数型言語の標準）
        const arg = this.check(TokenType.NOT)
          ? this.parseUnaryOnly()
          : this.primaryExpression()
        expr = new AST.FunctionApplication(
          expr,
          arg,
          this.previous().line,
          this.previous().column
        )
      } else {
        break
      }
    }

    return expr
  }

  // 次のトークンが式の開始になり得るかチェック
  private canStartExpression(): boolean {
    const type = this.peek().type
    return (
      type === TokenType.INTEGER ||
      type === TokenType.FLOAT ||
      type === TokenType.STRING ||
      type === TokenType.BOOLEAN ||
      type === TokenType.IDENTIFIER ||
      type === TokenType.PRINT ||
      type === TokenType.PUT_STR_LN ||
      type === TokenType.TO_STRING ||
      type === TokenType.HEAD ||
      type === TokenType.TAIL ||
      type === TokenType.LEFT_PAREN ||
      type === TokenType.LEFT_BRACKET ||
      type === TokenType.LEFT_BRACE ||
      type === TokenType.LAMBDA ||
      type === TokenType.NOT
    )
  }

  private primaryExpression(): AST.Expression {
    // Lambda expressions: \x -> expr or \x :Type -> expr
    if (this.match(TokenType.LAMBDA)) {
      return this.lambdaExpression()
    }

    if (this.match(TokenType.INTEGER)) {
      const value = parseInt(this.previous().value)
      return new AST.Literal(
        value,
        "integer",
        this.previous().line,
        this.previous().column
      )
    }

    if (this.match(TokenType.FLOAT)) {
      const value = parseFloat(this.previous().value)
      return new AST.Literal(
        value,
        "float",
        this.previous().line,
        this.previous().column
      )
    }

    if (this.match(TokenType.STRING)) {
      const value = this.previous().value
      return new AST.Literal(
        value,
        "string",
        this.previous().line,
        this.previous().column
      )
    }

    if (this.match(TokenType.BOOLEAN)) {
      const value = this.previous().value === "True"
      return new AST.Literal(
        value,
        "boolean",
        this.previous().line,
        this.previous().column
      )
    }

    // ビルトイン関数
    if (
      this.match(TokenType.PRINT, TokenType.PUT_STR_LN, TokenType.TO_STRING, TokenType.HEAD, TokenType.TAIL)
    ) {
      const functionName = this.previous().value as
        | "print"
        | "putStrLn"
        | "toString"
        | "head"
        | "tail"
      const line = this.previous().line
      const column = this.previous().column

      // 括弧付きの場合（後方互換性）
      if (this.check(TokenType.LEFT_PAREN)) {
        this.advance() // consume '('
        const args: AST.Expression[] = []

        if (!this.check(TokenType.RIGHT_PAREN)) {
          do {
            args.push(this.expression())
          } while (this.match(TokenType.COMMA))
        }

        this.consume(TokenType.RIGHT_PAREN, "Expected ')' after arguments")
        return new AST.BuiltinFunctionCall(functionName, args, line, column)
      } else {
        // 括弧なしの場合はIdentifierとして返し、callExpressionで処理
        return new AST.Identifier(functionName, line, column)
      }
    }

    if (this.match(TokenType.IDENTIFIER)) {
      const name = this.previous().value
      const line = this.previous().line
      const column = this.previous().column

      // 大文字で始まる場合はコンストラクタまたは構造体インスタンスの可能性
      if (name[0] === name[0].toUpperCase()) {
        // 構造体インスタンス化: Person { name: "Alice", age: 30 }
        if (this.check(TokenType.LEFT_BRACE)) {
          this.advance() // consume '{'
          const fields: (AST.RecordInitField | AST.RecordSpreadField)[] = []

          if (!this.check(TokenType.RIGHT_BRACE)) {
            do {
              this.skipNewlines()

              // Check for spread syntax
              if (this.match(TokenType.SPREAD)) {
                const spreadLine = this.previous().line
                const spreadColumn = this.previous().column
                const spreadExpr = this.expression()
                const spreadAst = new AST.SpreadExpression(
                  spreadExpr,
                  spreadLine,
                  spreadColumn
                )
                fields.push(
                  new AST.RecordSpreadField(spreadAst, spreadLine, spreadColumn)
                )
              } else {
                // Check for shorthand property notation
                const fieldToken = this.peek()
                if (fieldToken.type === TokenType.IDENTIFIER) {
                  const fieldName = this.advance().value
                  const fieldLine = this.previous().line
                  const fieldColumn = this.previous().column

                  this.skipNewlines()

                  // Check if this is shorthand notation (no colon follows)
                  if (
                    this.check(TokenType.COMMA) ||
                    this.check(TokenType.RIGHT_BRACE)
                  ) {
                    // Shorthand: Person { name, age }
                    fields.push(
                      new AST.RecordShorthandField(
                        fieldName,
                        fieldLine,
                        fieldColumn
                      )
                    )
                  } else {
                    // Regular field: Person { name: value }
                    this.consume(
                      TokenType.COLON,
                      "Expected ':' after field name"
                    )
                    this.skipNewlines()
                    const fieldValue = this.expression()

                    fields.push(
                      new AST.RecordInitField(
                        fieldName,
                        fieldValue,
                        fieldLine,
                        fieldColumn
                      )
                    )
                  }
                } else {
                  throw new ParseError("Expected field name", fieldToken)
                }
              }
              this.skipNewlines()
            } while (this.match(TokenType.COMMA))
          }

          this.skipNewlines()
          this.consume(
            TokenType.RIGHT_BRACE,
            "Expected '}' after struct fields"
          )

          return new AST.StructExpression(name, fields, line, column)
        }

        // 通常のコンストラクタ引数がある場合
        if (
          this.checkPrimaryStart() &&
          !this.check(TokenType.ARROW) &&
          !this.check(TokenType.PIPE)
        ) {
          const args: AST.Expression[] = []

          // 関数呼び出し風の括弧付きの場合: Cons(1, 2)
          // ただし、Cons (expr) のような場合は除外するため、先読みして判定
          if (
            this.check(TokenType.LEFT_PAREN) &&
            this.isActualFunctionCallSyntax()
          ) {
            this.advance() // consume '('

            if (!this.check(TokenType.RIGHT_PAREN)) {
              do {
                args.push(this.expression())
              } while (this.match(TokenType.COMMA))
            }

            this.consume(
              TokenType.RIGHT_PAREN,
              "Expected ')' after constructor arguments"
            )
          } else {
            // 関数型言語風の括弧なしの場合: Cons 1 2 または Cons (expr) (expr)
            let argCount = 0
            const maxArgs =
              name === "Cons"
                ? 2
                : name === "Just" || name === "Left" || name === "Right"
                  ? 1
                  : 0

            while (
              argCount < maxArgs &&
              this.checkPrimaryStart() &&
              !this.check(TokenType.ARROW) &&
              !this.check(TokenType.PIPE) &&
              !this.check(TokenType.NEWLINE) &&
              !this.check(TokenType.EOF) &&
              !this.check(TokenType.RIGHT_PAREN) &&
              !this.check(TokenType.RIGHT_BRACE) &&
              !this.check(TokenType.COMMA) &&
              !this.check(TokenType.SEMICOLON) &&
              !this.check(TokenType.LET) &&
              !this.check(TokenType.FN)
            ) {
              args.push(this.primaryExpression())
              argCount++
            }
          }

          return new AST.ConstructorExpression(name, args, line, column)
        }

        // 引数がない場合（Nothing, Empty等）はそのままコンストラクタ
        // ただし、引数が必要なコンストラクタ（Just, Left, Right, Cons）は除外
        if (name === "Nothing" || name === "Empty") {
          return new AST.ConstructorExpression(name, [], line, column)
        }
      }

      return new AST.Identifier(name, line, column)
    }

    if (this.match(TokenType.LEFT_PAREN)) {
      const line = this.previous().line
      const column = this.previous().column

      // Check for empty tuple () - not allowed in our design
      if (this.check(TokenType.RIGHT_PAREN)) {
        throw new ParseError("Empty tuples are not supported", this.peek())
      }

      const firstExpr = this.expression()

      // Check if this is a tuple (has comma)
      if (this.match(TokenType.COMMA)) {
        const elements = [firstExpr]

        // Parse remaining tuple elements
        do {
          elements.push(this.expression())
        } while (this.match(TokenType.COMMA))

        this.consume(TokenType.RIGHT_PAREN, "Expected ')' after tuple elements")

        // Enforce minimum 2 elements for tuples
        if (elements.length < 2) {
          throw new ParseError(
            "Tuples must have at least 2 elements",
            this.previous()
          )
        }

        return new AST.TupleExpression(elements, line, column)
      }

      // Single expression in parentheses
      this.consume(TokenType.RIGHT_PAREN, "Expected ')' after expression")
      return firstExpr
    }

    if (this.match(TokenType.LEFT_BRACKET)) {
      // Array literal [1, 2, 3] or List comprehension [x * 2 | x <- range, filter]
      const line = this.previous().line
      const column = this.previous().column

      if (this.check(TokenType.RIGHT_BRACKET)) {
        // Empty array []
        this.advance()
        return new AST.ArrayLiteral([], line, column)
      }

      this.skipNewlines()
      const firstExpr = this.expressionWithoutPipeline()
      this.skipNewlines()

      // Check if this is a list comprehension (has |)
      if (this.check(TokenType.PIPE)) {
        this.advance() // consume |
        return this.parseListComprehension(firstExpr, line, column)
      }

      // Regular array literal
      const elements: AST.Expression[] = [firstExpr]

      while (this.match(TokenType.COMMA)) {
        this.skipNewlines()
        elements.push(this.expression())
        this.skipNewlines()
      }

      this.consume(TokenType.RIGHT_BRACKET, "Expected ']' after array elements")
      return new AST.ArrayLiteral(elements, line, column)
    }

    if (this.match(TokenType.BACKTICK)) {
      // List sugar `[1, 2, 3] or `[] or list comprehension `[x * 2 | x <- range]
      const line = this.previous().line
      const column = this.previous().column

      this.consume(TokenType.LEFT_BRACKET, "Expected '[' after '`'")

      if (this.check(TokenType.RIGHT_BRACKET)) {
        // Empty list `[]
        this.advance()
        return new AST.ListSugar([], line, column)
      }

      this.skipNewlines()
      const firstExpr = this.expressionWithoutPipeline()
      this.skipNewlines()

      // Check if this is a list comprehension (has |)
      if (this.check(TokenType.PIPE)) {
        this.advance() // consume |
        const listComp = this.parseListComprehension(firstExpr, line, column)
        // Wrap in ListComprehensionSugar to distinguish from array comprehension
        return new AST.ListComprehensionSugar(
          listComp.expression,
          listComp.generators,
          listComp.filters,
          line,
          column
        )
      }

      // Regular list literal
      const elements: AST.Expression[] = [firstExpr]

      while (this.match(TokenType.COMMA)) {
        this.skipNewlines()
        elements.push(this.expression())
        this.skipNewlines()
      }

      this.consume(TokenType.RIGHT_BRACKET, "Expected ']' after list elements")
      return new AST.ListSugar(elements, line, column)
    }

    if (this.match(TokenType.LEFT_BRACE)) {
      // Record literal { name: "John", age: 30, ...other }
      const fields: (AST.RecordInitField | AST.RecordSpreadField)[] = []
      const line = this.previous().line
      const column = this.previous().column

      if (!this.check(TokenType.RIGHT_BRACE)) {
        do {
          this.skipNewlines()

          // Check for spread syntax
          if (this.match(TokenType.SPREAD)) {
            const spreadLine = this.previous().line
            const spreadColumn = this.previous().column
            const spreadExpr = this.expression()
            const spreadAst = new AST.SpreadExpression(
              spreadExpr,
              spreadLine,
              spreadColumn
            )
            fields.push(
              new AST.RecordSpreadField(spreadAst, spreadLine, spreadColumn)
            )
          } else {
            // Check for shorthand property notation
            const fieldToken = this.peek()
            if (fieldToken.type === TokenType.IDENTIFIER) {
              const fieldName = this.advance().value
              const fieldLine = this.previous().line
              const fieldColumn = this.previous().column

              this.skipNewlines()

              // Check if this is shorthand notation (no colon follows)
              if (
                this.check(TokenType.COMMA) ||
                this.check(TokenType.RIGHT_BRACE)
              ) {
                // Shorthand: { name, age }
                fields.push(
                  new AST.RecordShorthandField(
                    fieldName,
                    fieldLine,
                    fieldColumn
                  )
                )
              } else {
                // Regular field: { name: value }
                this.consume(TokenType.COLON, "Expected ':' after field name")
                this.skipNewlines()
                const fieldValue = this.expression()

                fields.push(
                  new AST.RecordInitField(
                    fieldName,
                    fieldValue,
                    fieldLine,
                    fieldColumn
                  )
                )
              }
            } else {
              throw new ParseError("Expected field name", fieldToken)
            }
          }
          this.skipNewlines()
        } while (this.match(TokenType.COMMA))
      }

      this.skipNewlines()

      this.consume(TokenType.RIGHT_BRACE, "Expected '}' after record fields")

      return new AST.RecordExpression(fields, line, column)
    }

    throw new ParseError("Expected expression", this.peek())
  }

  // =============================================================================
  // Utility Methods
  // =============================================================================

  private match(...types: TokenType[]): boolean {
    for (const type of types) {
      if (this.check(type)) {
        this.advance()
        return true
      }
    }
    return false
  }

  private check(type: TokenType): boolean {
    if (this.isAtEnd()) return false
    return this.peek().type === type
  }

  private advance(): Token {
    if (!this.isAtEnd()) this.current++
    return this.previous()
  }

  private isAtEnd(): boolean {
    return this.peek().type === TokenType.EOF
  }

  private peek(): Token {
    return this.tokens[this.current]
  }

  private previous(): Token {
    return this.tokens[this.current - 1]
  }

  private consume(type: TokenType, message: string): Token {
    if (this.check(type)) return this.advance()

    throw new ParseError(message, this.peek())
  }

  private skipNewlines(): void {
    while (this.check(TokenType.NEWLINE) || this.check(TokenType.COMMENT)) {
      this.advance()
    }
  }

  private checkPrimaryStart(): boolean {
    const type = this.peek().type
    return (
      type === TokenType.INTEGER ||
      type === TokenType.FLOAT ||
      type === TokenType.STRING ||
      type === TokenType.BOOLEAN ||
      type === TokenType.IDENTIFIER ||
      type === TokenType.PRINT ||
      type === TokenType.PUT_STR_LN ||
      type === TokenType.TO_STRING ||
      type === TokenType.HEAD ||
      type === TokenType.TAIL ||
      type === TokenType.LEFT_PAREN ||
      type === TokenType.LEFT_BRACKET ||
      type === TokenType.LEFT_BRACE ||
      type === TokenType.LAMBDA
    )
  }

  private isUpperCaseIdentifier(): boolean {
    if (this.peek().type !== TokenType.IDENTIFIER) {
      return false
    }
    const value = this.peek().value
    return value.length > 0 && value[0] === value[0].toUpperCase()
  }

  private isActualFunctionCallSyntax(): boolean {
    // 先読みして関数呼び出し構文かどうかを判定
    // Cons(1, 2) は true, Cons (expr) は false
    if (!this.check(TokenType.LEFT_PAREN)) {
      return false
    }

    // 現在位置を保存
    const saved = this.current

    try {
      this.advance() // consume '('

      // 空の括弧の場合は関数呼び出し
      if (this.check(TokenType.RIGHT_PAREN)) {
        return true
      }

      // 最初の式をスキップ
      this.skipExpression()

      // コンマがあれば関数呼び出し、なければ単なる括弧付き式
      const hasComma = this.check(TokenType.COMMA)
      return hasComma
    } catch {
      // パースエラーの場合は関数呼び出しと仮定
      return true
    } finally {
      // 位置を復元
      this.current = saved
    }
  }

  private skipExpression(): void {
    // 簡単な式のスキップ（完全ではないが、このコンテキストでは十分）
    let depth = 0

    while (!this.isAtEnd()) {
      const token = this.peek()

      if (token.type === TokenType.LEFT_PAREN) {
        depth++
      } else if (token.type === TokenType.RIGHT_PAREN) {
        if (depth === 0) break // 最上位の右括弧
        depth--
      } else if (depth === 0 && token.type === TokenType.COMMA) {
        break
      }

      this.advance()
    }
  }

  private blockExpression(): AST.BlockExpression {
    const statements: AST.Statement[] = []
    let returnExpression: AST.Expression | undefined

    while (!this.check(TokenType.RIGHT_BRACE) && !this.isAtEnd()) {
      // Skip newlines and comments
      if (this.match(TokenType.NEWLINE, TokenType.COMMENT)) {
        continue
      }

      // Check for explicit return statement
      if (this.check(TokenType.RETURN)) {
        const returnStmt = this.returnStatement()
        returnExpression = returnStmt.expression
        break
      }

      // Parse statement or expression
      const stmt = this.statement()
      if (stmt) {
        // If this is an expression statement and it's the last thing before closing brace,
        // treat it as the return expression
        if (stmt instanceof AST.ExpressionStatement) {
          // Look ahead to see if we're at the end of the block
          this.skipNewlines()
          if (this.check(TokenType.RIGHT_BRACE)) {
            returnExpression = stmt.expression
          } else {
            statements.push(stmt)
          }
        } else {
          statements.push(stmt)
        }
      }
    }

    this.consume(TokenType.RIGHT_BRACE, "Expected '}' after block")

    return new AST.BlockExpression(
      statements,
      returnExpression,
      this.previous().line,
      this.previous().column
    )
  }

  private lambdaExpression(): AST.LambdaExpression {
    const startLine = this.previous().line
    const startColumn = this.previous().column

    const parameters: AST.Parameter[] = []

    // Parse parameter(s) - support both single param and nested lambdas
    // \x -> expr  or  \x :Type -> expr
    do {
      const paramName = this.consume(
        TokenType.IDENTIFIER,
        "Expected parameter name after '\\'"
      ).value

      let paramType: AST.Type | undefined

      // Check for optional type annotation
      if (this.match(TokenType.COLON)) {
        paramType = this.parseType()
      } else {
        // Create a placeholder type that will be inferred
        paramType = new AST.PrimitiveType("_", startLine, startColumn)
      }

      parameters.push(
        new AST.Parameter(paramName, paramType, startLine, startColumn)
      )

      // If we find another lambda, it's a nested lambda (\a -> \b -> ...)
      if (this.check(TokenType.LAMBDA)) {
        break
      }
    } while (false) // Only parse one parameter per lambda

    this.consume(TokenType.ARROW, "Expected '->' after lambda parameter")

    // Parse the body - could be another lambda or expression
    const body = this.expression()

    return new AST.LambdaExpression(parameters, body, startLine, startColumn)
  }

  private checkStatementStart(): boolean {
    const type = this.peek().type
    return (
      type === TokenType.FN ||
      type === TokenType.EFFECTFUL ||
      type === TokenType.TYPE ||
      type === TokenType.IMPL ||
      type === TokenType.IMPORT ||
      type === TokenType.LET
    )
  }

  private isConstructorName(name: string): boolean {
    // Constructor names start with uppercase letter
    return name[0] === name[0].toUpperCase()
  }

  private checkAhead(type: TokenType): boolean {
    // Look ahead to the next token without consuming
    let lookahead = this.current + 1

    // Skip whitespace and newlines
    while (
      lookahead < this.tokens.length &&
      (this.tokens[lookahead].type === TokenType.NEWLINE ||
        this.tokens[lookahead].type === TokenType.WHITESPACE)
    ) {
      lookahead++
    }

    return (
      lookahead < this.tokens.length && this.tokens[lookahead].type === type
    )
  }

  private parseListComprehension(
    expression: AST.Expression,
    line: number,
    column: number
  ): AST.ListComprehension {
    // Parse generators: x <- range, y <- list
    const generators: AST.Generator[] = []
    const filters: AST.Expression[] = []

    do {
      this.skipNewlines()

      // Check if this is a generator or filter
      // Need to check if we have "IDENTIFIER GENERATOR" pattern
      if (this.check(TokenType.IDENTIFIER)) {
        // Look ahead to see if there's a GENERATOR token
        let lookahead = this.current + 1
        while (
          lookahead < this.tokens.length &&
          (this.tokens[lookahead].type === TokenType.NEWLINE ||
            this.tokens[lookahead].type === TokenType.WHITESPACE)
        ) {
          lookahead++
        }

        if (
          lookahead < this.tokens.length &&
          this.tokens[lookahead].type === TokenType.GENERATOR
        ) {
          // Generator: x <- range
          const variable = this.consume(
            TokenType.IDENTIFIER,
            "Expected variable name"
          ).value
          this.consume(TokenType.GENERATOR, "Expected '<-' in generator")
          const iterable = this.expression()

          generators.push(new AST.Generator(variable, iterable, line, column))
        } else {
          // Filter: x % 2 == 0
          const filterExpr = this.expression()
          filters.push(filterExpr)
        }
      } else {
        // Filter: expression
        const filterExpr = this.expression()
        filters.push(filterExpr)
      }

      this.skipNewlines()
    } while (this.match(TokenType.COMMA))

    this.consume(
      TokenType.RIGHT_BRACKET,
      "Expected ']' after list comprehension"
    )

    return new AST.ListComprehension(
      expression,
      generators,
      filters,
      line,
      column
    )
  }

  // メソッド呼び出しかどうかを判定
  private isMethodCall(): boolean {
    // 現在の位置を保存
    const savedPosition = this.current

    // 次のトークンがIDENTIFIERかチェック
    if (!this.check(TokenType.IDENTIFIER)) {
      return false
    }

    this.advance()

    // その後に式が続くかチェック
    const hasArgument = this.canStartExpression()

    // 位置を復元
    this.current = savedPosition

    return hasArgument
  }

  // Context-aware method call detection
  private isActualMethodCall(receiver: AST.Expression): boolean {
    // Check if the next identifier is a known method name from any struct impl block
    if (!this.check(TokenType.IDENTIFIER)) {
      return false
    }

    const methodName = this.peek().value

    // Check if this method name exists in any registered impl block
    for (const methodSet of this.methodRegistry.values()) {
      if (methodSet.has(methodName)) {
        // This is a known method name, so allow method call parsing
        // The type checker will later verify the receiver type matches
        return true
      }
    }

    // If the method name is not registered, treat as function application
    return false
  }

  private recordPattern(): AST.RecordPattern {
    this.consume(TokenType.LEFT_BRACE, "Expected '{' to start record pattern")
    const fields: AST.RecordPatternField[] = []

    if (!this.check(TokenType.RIGHT_BRACE)) {
      do {
        this.skipNewlines()
        const fieldName = this.consume(
          TokenType.IDENTIFIER,
          "Expected field name"
        ).value

        let alias: string | undefined
        // Check for alias syntax: {x: posX}
        if (this.match(TokenType.COLON)) {
          alias = this.consume(
            TokenType.IDENTIFIER,
            "Expected alias name"
          ).value
        }

        fields.push(
          new AST.RecordPatternField(
            fieldName,
            this.previous().line,
            this.previous().column,
            alias
          )
        )
        this.skipNewlines()
      } while (this.match(TokenType.COMMA))
    }

    this.consume(TokenType.RIGHT_BRACE, "Expected '}' to close record pattern")
    return new AST.RecordPattern(
      fields,
      this.previous().line,
      this.previous().column
    )
  }

  private structPattern(structName: string): AST.StructPattern {
    this.consume(TokenType.LEFT_BRACE, "Expected '{' to start struct pattern")
    const fields: AST.RecordPatternField[] = []

    if (!this.check(TokenType.RIGHT_BRACE)) {
      do {
        this.skipNewlines()
        const fieldName = this.consume(
          TokenType.IDENTIFIER,
          "Expected field name"
        ).value

        let alias: string | undefined
        // Check for alias syntax: {x: posX}
        if (this.match(TokenType.COLON)) {
          alias = this.consume(
            TokenType.IDENTIFIER,
            "Expected alias name"
          ).value
        }

        fields.push(
          new AST.RecordPatternField(
            fieldName,
            this.previous().line,
            this.previous().column,
            alias
          )
        )
        this.skipNewlines()
      } while (this.match(TokenType.COMMA))
    }

    this.consume(TokenType.RIGHT_BRACE, "Expected '}' to close struct pattern")
    return new AST.StructPattern(
      structName,
      fields,
      this.previous().line,
      this.previous().column
    )
  }
}

// Convenience function for parsing
export function parse(tokens: Token[]): ParseResult {
  const parser = new Parser(tokens)
  return parser.parse()
}
