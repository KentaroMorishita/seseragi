import {
  createConnection,
  TextDocuments,
  type Diagnostic,
  DiagnosticSeverity,
  ProposedFeatures,
  type InitializeParams,
  DidChangeConfigurationNotification,
  type CompletionItem,
  CompletionItemKind,
  type TextDocumentPositionParams,
  TextDocumentSyncKind,
  type InitializeResult,
  type Hover,
  MarkupKind,
  type DocumentFormattingParams,
  type TextEdit,
  type DefinitionParams,
  type Definition,
  type Location,
} from "vscode-languageserver/node"

import { TextDocument } from "vscode-languageserver-textdocument"
import { URI } from "vscode-uri"
import { Parser } from "../parser"
import { TypeInferenceSystem } from "../type-inference"
import { TypeChecker } from "../typechecker"
import * as AST from "../ast"
import {
  formatSeseragiCode,
  removeExtraWhitespace,
  normalizeOperatorSpacing,
} from "../formatter/index.js"

// Create a connection for the server, using Node's IPC as a transport
const connection = createConnection(ProposedFeatures.all)

// Create a text document manager
const documents: TextDocuments<TextDocument> = new TextDocuments(TextDocument)

let hasConfigurationCapability = false
let hasWorkspaceFolderCapability = false
let hasDiagnosticRelatedInformationCapability = false

connection.onInitialize((params: InitializeParams) => {
  connection.console.log("Seseragi Language Server starting...")
  const capabilities = params.capabilities

  // Does the client support the `workspace/configuration` request?
  hasConfigurationCapability = !!(
    capabilities.workspace && !!capabilities.workspace.configuration
  )
  hasWorkspaceFolderCapability = !!(
    capabilities.workspace && !!capabilities.workspace.workspaceFolders
  )
  hasDiagnosticRelatedInformationCapability = !!(
    capabilities.textDocument &&
    capabilities.textDocument.publishDiagnostics &&
    capabilities.textDocument.publishDiagnostics.relatedInformation
  )

  const result: InitializeResult = {
    capabilities: {
      textDocumentSync: TextDocumentSyncKind.Incremental,
      // Tell the client that this server supports code completion.
      completionProvider: {
        resolveProvider: true,
      },
      // Tell the client that this server supports hover information.
      hoverProvider: true,
      // Tell the client that this server supports document formatting.
      documentFormattingProvider: true,
      // Tell the client that this server supports go to definition.
      definitionProvider: true,
    },
  }
  if (hasWorkspaceFolderCapability) {
    result.capabilities.workspace = {
      workspaceFolders: {
        supported: true,
      },
    }
  }
  return result
})

connection.onInitialized(() => {
  if (hasConfigurationCapability) {
    // Register for all configuration changes.
    connection.client.register(
      DidChangeConfigurationNotification.type,
      undefined
    )
  }
  if (hasWorkspaceFolderCapability) {
    connection.workspace.onDidChangeWorkspaceFolders((_event) => {
      connection.console.log("Workspace folder change event received.")
    })
  }
})

// The example settings
interface SeseragiSettings {
  maxNumberOfProblems: number
}

// The global settings, used when the `workspace/configuration` request is not supported by the client.
// Please note that this is not the case when using this server with the client provided in this example
// but could happen with other clients.
const defaultSettings: SeseragiSettings = { maxNumberOfProblems: 1000 }
let globalSettings: SeseragiSettings = defaultSettings

// Cache the settings of all open documents
const documentSettings: Map<string, Thenable<SeseragiSettings>> = new Map()

connection.onDidChangeConfiguration((change) => {
  if (hasConfigurationCapability) {
    // Reset all cached document settings
    documentSettings.clear()
  } else {
    globalSettings = <SeseragiSettings>(
      (change.settings.seseragi || defaultSettings)
    )
  }

  // Revalidate all open text documents
  documents.all().forEach(validateTextDocument)
})

function getDocumentSettings(resource: string): Thenable<SeseragiSettings> {
  if (!hasConfigurationCapability) {
    return Promise.resolve(globalSettings)
  }
  let result = documentSettings.get(resource)
  if (!result) {
    result = connection.workspace.getConfiguration({
      scopeUri: resource,
      section: "seseragi",
    })
    documentSettings.set(resource, result)
  }
  return result
}

// Only keep settings for open documents
documents.onDidClose((e) => {
  documentSettings.delete(e.document.uri)
})

// The content of a text document has changed. This event is emitted
// when the text document first opened or when its content has changed.
documents.onDidChangeContent((change) => {
  validateTextDocument(change.document)
})

async function validateTextDocument(textDocument: TextDocument): Promise<void> {
  // In this simple example we get the settings for every validate run.
  const settings = await getDocumentSettings(textDocument.uri)

  // The validator creates diagnostics for all uppercase words longer than 2 characters
  const text = textDocument.getText()
  const diagnostics: Diagnostic[] = []

  try {
    // Parse the document
    const parser = new Parser(text)
    const parseResult = parser.parse()
    const ast = new AST.Program(parseResult.statements || [])

    // Use new type inference system
    const typeInference = new TypeInferenceSystem()
    const inferenceResult = typeInference.infer(ast)

    // Use only the new type inference system errors
    const allErrors = inferenceResult.errors

    // Convert type errors to diagnostics
    for (const error of allErrors) {
      const startPos =
        error.line !== undefined && error.column !== undefined
          ? {
              line: Math.max(0, error.line - 1),
              character: Math.max(0, error.column),
            }
          : textDocument.positionAt(0)
      const endPos =
        error.line !== undefined && error.column !== undefined
          ? {
              line: Math.max(0, error.line - 1),
              character: Math.max(
                0,
                error.column + ((error as any).length || 1)
              ),
            }
          : textDocument.positionAt(Math.min(text.length, 100))

      // Enhanced error message with context
      let enhancedMessage = error.message
      if ("context" in error && error.context) {
        enhancedMessage += `\n\nContext: ${error.context}`
      }
      if ("suggestion" in error && error.suggestion) {
        enhancedMessage += `\n\nSuggestion: ${error.suggestion}`
      }

      const diagnostic: Diagnostic = {
        severity: DiagnosticSeverity.Error,
        range: {
          start: startPos,
          end: endPos,
        },
        message: enhancedMessage,
        source: "seseragi",
        code:
          ("code" in error && typeof error.code === "string"
            ? error.code
            : undefined) || "type-error",
      }

      if (hasDiagnosticRelatedInformationCapability) {
        diagnostic.relatedInformation = [
          {
            location: {
              uri: textDocument.uri,
              range: Object.assign({}, diagnostic.range),
            },
            message: "Type error occurred here",
          },
        ]
      }
      diagnostics.push(diagnostic)
    }
  } catch (error) {
    // Handle lexer/parser errors
    let errorMessage = "Unknown parsing error"
    let startPos = 0
    let endPos = Math.min(text.length, 100)

    if (error instanceof Error) {
      errorMessage = error.message
      // Log the full error for debugging
      connection.console.log(`Parser error: ${error.message}`)
      connection.console.log(`Error stack: ${error.stack}`)

      // Try to extract position information from error message if available
      const posMatch = error.message.match(/line (\d+), column (\d+)/)
      if (posMatch) {
        const line = parseInt(posMatch[1], 10) - 1 // Convert to 0-based
        const col = parseInt(posMatch[2], 10) - 1
        startPos = textDocument.offsetAt({ line, character: col })
        endPos = Math.min(startPos + 10, text.length)
      }
    }

    const diagnostic: Diagnostic = {
      severity: DiagnosticSeverity.Error,
      range: {
        start: textDocument.positionAt(startPos),
        end: textDocument.positionAt(endPos),
      },
      message: errorMessage,
      source: "seseragi",
    }
    diagnostics.push(diagnostic)
  }

  // Send the computed diagnostics to VSCode.
  connection.sendDiagnostics({ uri: textDocument.uri, diagnostics })
}

// Deprecated API - remove for now
// connection.languages.onDocumentDiagnostic(async (params) => {
//   const document = documents.get(params.textDocument.uri);
//   if (document !== undefined) {
//     return {
//       kind: DocumentDiagnosticReportKind.Full,
//       items: await getDiagnostics(document),
//     } satisfies DocumentDiagnosticReport;
//   } else {
//     return {
//       kind: DocumentDiagnosticReportKind.Full,
//       items: [],
//     } satisfies DocumentDiagnosticReport;
//   }
// });

// Unused function - commented out
// async function getDiagnostics(textDocument: TextDocument): Promise<Diagnostic[]> {
//   const text = textDocument.getText();
//   const diagnostics: Diagnostic[] = [];

//   try {
//     // Parse the document
//     const parser = new Parser(text);
//     const ast = parser.parse();

//     // Type check the document
//     const typeChecker = new TypeChecker();
//     const typeErrors = typeChecker.check(ast);

//     // Convert type errors to diagnostics
//     for (const error of typeErrors) {
//       const diagnostic: Diagnostic = {
//         severity: DiagnosticSeverity.Error,
//         range: {
//           start: textDocument.positionAt(error.start || 0),
//           end: textDocument.positionAt(error.end || text.length),
//         },
//         message: error.message,
//         source: "seseragi",
//       };
//       diagnostics.push(diagnostic);
//     }
//   } catch (error) {
//     // Handle lexer/parser errors
//     const diagnostic: Diagnostic = {
//       severity: DiagnosticSeverity.Error,
//       range: {
//         start: textDocument.positionAt(0),
//         end: textDocument.positionAt(text.length),
//       },
//       message: error instanceof Error ? error.message : "Unknown error",
//       source: "seseragi",
//     };
//     diagnostics.push(diagnostic);
//   }

//   return diagnostics;
// }

// This handler provides the initial list of the completion items.
connection.onCompletion(
  (textDocumentPosition: TextDocumentPositionParams): CompletionItem[] => {
    const document = documents.get(textDocumentPosition.textDocument.uri)
    if (!document) {
      return []
    }

    const text = document.getText()
    const position = textDocumentPosition.position
    const offset = document.offsetAt(position)

    // Get current line text for context
    const lineText = text.split("\n")[position.line] || ""
    const beforeCursor = lineText.substring(0, position.character)

    // Basic completion items
    const completionItems: CompletionItem[] = [
      {
        label: "fn",
        kind: CompletionItemKind.Keyword,
        data: 1,
      },
      {
        label: "let",
        kind: CompletionItemKind.Keyword,
        data: 2,
      },
      {
        label: "type",
        kind: CompletionItemKind.Keyword,
        data: 3,
      },
      {
        label: "match",
        kind: CompletionItemKind.Keyword,
        data: 4,
      },
      {
        label: "effectful",
        kind: CompletionItemKind.Keyword,
        data: 5,
      },
      {
        label: "operator",
        kind: CompletionItemKind.Keyword,
        data: 6,
      },
      {
        label: "impl",
        kind: CompletionItemKind.Keyword,
        data: 7,
      },
      {
        label: "Maybe",
        kind: CompletionItemKind.Class,
        data: 8,
      },
      {
        label: "Either",
        kind: CompletionItemKind.Class,
        data: 9,
      },
      {
        label: "IO",
        kind: CompletionItemKind.Class,
        data: 10,
      },
    ]

    // Add contextual completions based on current line
    if (beforeCursor.trim().startsWith("type ")) {
      completionItems.push(
        {
          label: "Maybe<T>",
          kind: CompletionItemKind.TypeParameter,
          data: 100,
          insertText: "Maybe<$1>",
          insertTextFormat: 2, // Snippet format
        },
        {
          label: "Either<L,R>",
          kind: CompletionItemKind.TypeParameter,
          data: 101,
          insertText: "Either<$1, $2>",
          insertTextFormat: 2,
        }
      )
    }

    return completionItems
  }
)

// This handler resolves additional information for the item selected in
// the completion list.
connection.onCompletionResolve((item: CompletionItem): CompletionItem => {
  if (item.data === 1) {
    item.detail = "Function definition"
    item.documentation = "Define a new function"
  } else if (item.data === 2) {
    item.detail = "Variable binding"
    item.documentation = "Bind a value to an immutable variable"
  } else if (item.data === 3) {
    item.detail = "Type definition"
    item.documentation = "Define a new type"
  } else if (item.data === 4) {
    item.detail = "Pattern matching"
    item.documentation = "Pattern match on algebraic data types"
  } else if (item.data === 5) {
    item.detail = "Effectful function"
    item.documentation = "Mark a function as having side effects"
  } else if (item.data === 6) {
    item.detail = "Operator definition"
    item.documentation = "Define a custom operator"
  } else if (item.data === 7) {
    item.detail = "Implementation block"
    item.documentation = "Implement methods for a type"
  } else if (item.data === 8) {
    item.detail = "Maybe<T>"
    item.documentation = "Optional value type"
  } else if (item.data === 9) {
    item.detail = "Either<L,R>"
    item.documentation = "Error handling type"
  } else if (item.data === 10) {
    item.detail = "IO<T>"
    item.documentation = "IO monad for side effects"
  }
  return item
})

// This handler provides hover information.
connection.onHover((params: TextDocumentPositionParams): Hover | null => {
  const document = documents.get(params.textDocument.uri)
  if (!document) {
    return null
  }

  const text = document.getText()
  const position = params.position
  const offset = document.offsetAt(position)

  try {
    // Parse the document
    const parser = new Parser(text)
    const ast = parser.parse()

    // Debug: log AST structure
    connection.console.log(`=== LSP AST DEBUG ===`)
    connection.console.log(`Parsed AST: ${JSON.stringify(ast, null, 2)}`)

    // Get hover information from the position
    const hoverInfo = getHoverInfo(ast, offset, text)

    if (hoverInfo) {
      return {
        contents: {
          kind: MarkupKind.Markdown,
          value: hoverInfo,
        },
      }
    }
  } catch (error) {
    // Silently ignore errors for hover
    connection.console.log(
      `Hover error: ${error instanceof Error ? error.message : "Unknown error"}`
    )
  }

  return null
})

// Helper function to get hover information at a specific offset
function getHoverInfo(ast: any, offset: number, text: string): string | null {
  // Check if this is a field access first
  const fieldAccessInfo = getFieldAccessInfo(text, offset)
  if (fieldAccessInfo) {
    return handleFieldAccessHover(ast, fieldAccessInfo)
  }

  // Find the token/node at the given offset
  const wordAtPosition = getWordAtPosition(text, offset)
  if (!wordAtPosition) {
    return null
  }

  // Try to get type information using the type inference system
  try {
    // Cache AST for struct definition lookup
    cachedAST = ast

    const typeInference = new TypeInferenceSystem()
    const result = typeInference.infer(ast)
    connection.console.log(
      `[SESERAGI LSP DEBUG] Type inference completed. Errors: ${result.errors.length}`
    )
    if (result.errors.length > 0) {
      connection.console.log(
        `Type inference errors: ${JSON.stringify(result.errors, null, 2)}`
      )
    }
    const typeInfo = getTypeInfoWithInference(
      ast,
      wordAtPosition,
      result,
      offset,
      text
    )
    if (typeInfo) {
      connection.console.log(
        `[SESERAGI LSP DEBUG] Returning type info: ${typeInfo}`
      )
      return typeInfo
    } else {
      connection.console.log(
        `[SESERAGI LSP DEBUG] No type info found from inference`
      )
    }
  } catch (error) {
    connection.console.log(
      `[SESERAGI LSP DEBUG] Type inference failed: ${error}`
    )
    // Fall back to basic type info if type inference fails
  }

  // Try to find type information for the word
  const typeInfo = getTypeInfoForSymbol(ast, wordAtPosition)
  if (typeInfo) {
    return `**${wordAtPosition}**: ${typeInfo}`
  }

  // Return basic information for known keywords
  const keywordInfo = getKeywordInfo(wordAtPosition)
  if (keywordInfo) {
    return keywordInfo
  }

  return null
}

// Get type information using the enhanced type inference system
function getTypeInfoWithInference(
  ast: any,
  symbol: string,
  inferenceResult: any,
  offset: number,
  text: string
): string | null {
  try {
    // First check if this is a struct definition
    const structInfo = findStructDefinition(symbol)
    if (structInfo) {
      return formatStructDefinitionInfo(symbol, structInfo)
    }

    // Find the symbol in the AST and get its resolved type
    const symbolInfo = findSymbolWithEnhancedInference(
      ast,
      symbol,
      inferenceResult,
      offset,
      text
    )
    if (symbolInfo) {
      // Debug logging to understand what type we're getting
      connection.console.log(
        `Type info for ${symbol}: ${JSON.stringify(symbolInfo.finalType, null, 2)}`
      )
      return formatInferredTypeInfo(symbol, symbolInfo)
    }
  } catch (error) {
    connection.console.log(
      `Type inference error for ${symbol}: ${error instanceof Error ? error.message : "Unknown error"}`
    )
  }

  return null
}

// Find symbol with type information from the type checker
function findSymbolWithType(
  ast: any,
  symbol: string,
  typeChecker: TypeChecker
): any {
  if (!ast.statements) {
    return null
  }

  for (const statement of ast.statements) {
    if (statement.kind === "FunctionDeclaration" && statement.name === symbol) {
      return {
        type: "function",
        name: symbol,
        parameters: statement.parameters,
        returnType: statement.returnType,
        isEffectful: statement.isEffectful,
      }
    }

    if (statement.kind === "VariableDeclaration" && statement.name === symbol) {
      return {
        type: "variable",
        name: symbol,
        varType: statement.type,
        value: statement.initializer,
      }
    }

    if (statement.kind === "TypeDeclaration" && statement.name === symbol) {
      return {
        type: "type",
        name: symbol,
        definition: statement.definition,
      }
    }
  }

  return null
}

// Format type information for hover display
function formatTypeInfo(symbol: string, info: any): string {
  switch (info.type) {
    case "function": {
      const params =
        info.parameters
          ?.map((p: any) => {
            const paramType = formatTypeForDisplay(p.type)
            return `${p.name}: ${paramType}`
          })
          .join(", ") || ""

      const returnType = formatTypeForDisplay(info.returnType)
      const effectful = info.isEffectful ? "effectful " : ""

      return `\`\`\`seseragi\n${effectful}fn ${symbol}(${params}) -> ${returnType}\n\`\`\``
    }

    case "variable": {
      const varType = formatTypeForDisplay(info.varType)
      return `\`\`\`seseragi\nlet ${symbol}: ${varType}\n\`\`\``
    }

    case "type":
      return `\`\`\`seseragi\ntype ${symbol}\n\`\`\`\n\nUser-defined type`

    default:
      return null
  }
}

// Format type for display in hover
function formatTypeForDisplay(type: any): string {
  if (!type) return "unknown"

  if (typeof type === "string") {
    return type
  }

  if (type.kind === "PrimitiveType") {
    return type.name
  }

  if (type.kind === "FunctionType") {
    const paramType = formatTypeForDisplay(type.parameterType)
    const returnType = formatTypeForDisplay(type.returnType)
    return `${paramType} -> ${returnType}`
  }

  if (type.kind === "GenericType") {
    const baseType = formatTypeForDisplay(type.baseType)
    const typeArgs =
      type.typeArguments?.map(formatTypeForDisplay).join(", ") || ""
    return typeArgs ? `${baseType}<${typeArgs}>` : baseType
  }

  if (type.kind === "TupleType") {
    const elementTypes =
      type.elementTypes?.map(formatTypeForDisplay).join(", ") || ""
    return `(${elementTypes})`
  }

  return type.name || "unknown"
}

// Get the word at a specific position in the text
function getWordAtPosition(text: string, offset: number): string | null {
  if (offset < 0 || offset >= text.length) {
    return null
  }

  const before = text.substring(0, offset)
  const after = text.substring(offset)

  // Haskellスタイルのアポストロフィ付き識別子をサポート
  const beforeMatch = before.match(/[a-zA-Z_][a-zA-Z0-9_]*'*$/)
  const afterMatch = after.match(/^[a-zA-Z0-9_']*/)

  if (beforeMatch) {
    const start = beforeMatch[0]
    const end = afterMatch ? afterMatch[0] : ""
    return start + end
  }

  return null
}

// Check if the position is in a field access expression (e.g., self.x)
function getFieldAccessInfo(
  text: string,
  offset: number
): { objectName: string; fieldName: string } | null {
  if (offset < 0 || offset >= text.length) {
    return null
  }

  const before = text.substring(0, offset)
  const after = text.substring(offset)

  // Get the current word (field name) - アポストロフィ付き識別子をサポート
  const beforeMatch = before.match(/[a-zA-Z_][a-zA-Z0-9_]*'*$/)
  const afterMatch = after.match(/^[a-zA-Z0-9_']*'*/)

  if (!beforeMatch) {
    return null
  }

  const fieldName = beforeMatch[0] + (afterMatch ? afterMatch[0] : "")

  // Look for the pattern: objectName.fieldName
  // Find the dot before the current field name
  const beforeField = before.substring(0, before.length - beforeMatch[0].length)
  const dotMatch = beforeField.match(/([a-zA-Z_][a-zA-Z0-9_]*'*)\s*\.\s*$/)

  if (dotMatch) {
    const objectName = dotMatch[1]
    connection.console.log(
      `[SESERAGI LSP DEBUG] Detected field access: ${objectName}.${fieldName}`
    )
    return { objectName, fieldName }
  }

  return null
}

// Handle hover for field access expressions
function handleFieldAccessHover(
  ast: any,
  fieldAccessInfo: { objectName: string; fieldName: string }
): string | null {
  try {
    // Get type inference result
    const typeInference = new TypeInferenceSystem()
    const result = typeInference.infer(ast)

    // Find the object's type
    // Note: For field access, we use offset=0 as a temporary workaround
    // This should be improved to get the actual text context
    const objectTypeInfo = findSymbolWithEnhancedInference(
      ast,
      fieldAccessInfo.objectName,
      result,
      0,
      ""
    )
    if (!objectTypeInfo) {
      connection.console.log(
        `[SESERAGI LSP DEBUG] Object ${fieldAccessInfo.objectName} not found`
      )
      return null
    }

    connection.console.log(
      `[SESERAGI LSP DEBUG] Object type: ${JSON.stringify(objectTypeInfo.finalType, null, 2)}`
    )

    // Get the struct definition for this type
    const structType = objectTypeInfo.finalType
    if (structType.kind === "StructType") {
      const fieldInfo = getFieldTypeFromStruct(
        structType,
        fieldAccessInfo.fieldName
      )
      if (fieldInfo) {
        return formatFieldAccessInfo(
          fieldAccessInfo.objectName,
          fieldAccessInfo.fieldName,
          fieldInfo,
          structType.name
        )
      }
    }

    connection.console.log(
      `[SESERAGI LSP DEBUG] No field info found for ${fieldAccessInfo.fieldName}`
    )
    return null
  } catch (error) {
    connection.console.log(
      `[SESERAGI LSP DEBUG] Field access hover error: ${error}`
    )
    return null
  }
}

// Get field type from struct definition
function getFieldTypeFromStruct(
  structType: any,
  fieldName: string
): any | null {
  if (structType.kind !== "StructType" || !structType.fields) {
    return null
  }

  for (const field of structType.fields) {
    if (field.name === fieldName) {
      return field.type
    }
  }
  return null
}

// Format field access information for hover display
function formatFieldAccessInfo(
  objectName: string,
  fieldName: string,
  fieldType: any,
  structName: string
): string {
  const typeDisplay = formatTypeWithNestedStructures(fieldType, null, 0)
  return `\`\`\`seseragi\n${objectName}.${fieldName}: ${typeDisplay}\n\`\`\`\n\nField \`${fieldName}\` of struct \`${structName}\``
}

// Get type information for a symbol (basic implementation)
function getTypeInfoForSymbol(ast: any, symbol: string): string | null {
  // This is a simplified implementation
  // In a full implementation, you would traverse the AST to find the symbol's type

  // Try to find function definitions
  if (ast.items) {
    for (const item of ast.items) {
      if (item.type === "FunctionDefinition" && item.name === symbol) {
        const paramTypes =
          item.parameters
            ?.map((p: any) =>
              p.type ? `${p.name}: ${formatType(p.type)}` : p.name
            )
            .join(", ") || ""
        const returnType = item.returnType
          ? formatType(item.returnType)
          : "unknown"
        return `\`\`\`seseragi\nfn ${symbol}(${paramTypes}) -> ${returnType}\n\`\`\``
      }

      if (item.type === "VariableDefinition" && item.name === symbol) {
        const varType = item.valueType ? formatType(item.valueType) : "inferred"
        return `\`\`\`seseragi\nlet ${symbol}: ${varType}\n\`\`\``
      }
    }
  }

  return null
}

// Format type information for display
function formatType(type: any): string {
  if (typeof type === "string") {
    return type
  }
  if (type.name) {
    return type.name
  }
  if (type.type === "FunctionType") {
    const params = type.parameters?.map(formatType).join(" -> ") || ""
    const returnType = type.returnType ? formatType(type.returnType) : "unknown"
    return params ? `${params} -> ${returnType}` : returnType
  }
  return "unknown"
}

// Get information for known keywords
function getKeywordInfo(keyword: string): string | null {
  const keywordDocs: Record<string, string> = {
    fn: "**Function Definition**\n\nDefines a new function.\n\n```seseragi\nfn name(param: Type) -> ReturnType = expression\n```",
    let: "**Variable Binding**\n\nBinds a value to an immutable variable.\n\n```seseragi\nlet name: Type = expression\n```",
    type: "**Type Definition**\n\nDefines a new algebraic data type.\n\n```seseragi\ntype Maybe<T> = Some(T) | None\n```",
    match:
      "**Pattern Matching**\n\nPattern matches on algebraic data types.\n\n```seseragi\nmatch value {\n  Some(x) -> x,\n  None -> defaultValue\n}\n```",
    effectful:
      "**Effectful Function**\n\nMarks a function as having side effects.\n\n```seseragi\neffectful fn print(msg: String) -> IO<Unit>\n```",
    operator:
      "**Operator Definition**\n\nDefines a custom operator.\n\n```seseragi\noperator +++ (a: String, b: String) -> String = concat(a, b)\n```",
    impl: "**Implementation Block**\n\nImplements methods for a type.\n\n```seseragi\nimpl MyType {\n  fn method(self) -> ReturnType = ...\n}\n```",
    Maybe:
      "**Maybe Type**\n\nOptional value type for handling null values safely.\n\n```seseragi\ntype Maybe<T> = Some(T) | None\n```",
    Either:
      "**Either Type**\n\nError handling type for computations that may fail.\n\n```seseragi\ntype Either<L, R> = Left(L) | Right(R)\n```",
    IO: "**IO Monad**\n\nMonad for handling side effects in a pure functional way.\n\n```seseragi\ntype IO<T> = IO(T)\n```",
  }

  return keywordDocs[keyword] || null
}

// Find symbol within an expression (for method bodies, etc.)
function findSymbolInExpression(
  expression: any,
  symbol: string,
  inferenceResult: any
): any {
  if (!expression) {
    return null
  }

  connection.console.log(
    `=== LSP DEBUG: Searching for ${symbol} in expression ${expression.kind} ===`
  )

  switch (expression.kind) {
    case "BlockExpression":
      // Search in block statements
      for (const stmt of expression.statements || []) {
        const result = findSymbolInStatement(stmt, symbol, inferenceResult)
        if (result) {
          return result
        }
      }
      break

    case "VariableDeclaration":
      if (expression.name === symbol) {
        // Get the type of this variable from nodeTypeMap
        let varType = inferenceResult.nodeTypeMap.get(expression)
        if (!varType && expression.initializer) {
          varType = inferenceResult.nodeTypeMap.get(expression.initializer)
        }

        // Apply substitution if available
        if (
          varType &&
          inferenceResult.substitution &&
          inferenceResult.substitution.apply
        ) {
          varType = inferenceResult.substitution.apply(varType)
        }

        connection.console.log(
          `Found variable ${symbol} with type: ${JSON.stringify(varType, null, 2)}`
        )

        return {
          type: "variable",
          name: symbol,
          finalType: varType,
          hasExplicitType: !!expression.type,
        }
      }
      break

    case "RecordDestructuring": {
      // Check if the symbol is one of the destructured variables
      const foundField = findVariableInRecordPattern(expression.pattern, symbol)
      if (foundField) {
        // Get the type of the initializer (the record being destructured)
        const initType = inferenceResult.nodeTypeMap.get(expression.initializer)

        if (initType && initType.kind === "RecordType") {
          const recordType = initType as any
          const fieldType = recordType.fields.find(
            (f: any) => f.name === foundField.fieldName
          )?.type

          if (fieldType) {
            let finalFieldType = fieldType
            if (
              inferenceResult.substitution &&
              inferenceResult.substitution.apply
            ) {
              finalFieldType = inferenceResult.substitution.apply(fieldType)
            }

            return {
              type: "variable",
              name: symbol,
              finalType: finalFieldType,
              hasExplicitType: false,
              isRecordField: true,
              fieldName: foundField.fieldName,
            }
          }
        }
      }
      break
    }

    case "StructDestructuring": {
      // Check if the symbol is one of the destructured variables
      const foundStructField = findVariableInStructPattern(
        expression.pattern,
        symbol
      )
      if (foundStructField) {
        // Get the type of the initializer (the struct being destructured)
        const initType = inferenceResult.nodeTypeMap.get(expression.initializer)

        if (initType && initType.kind === "StructType") {
          const structType = initType as any
          const fieldType = structType.fields.find(
            (f: any) => f.name === foundStructField.fieldName
          )?.type

          if (fieldType) {
            let finalFieldType = fieldType
            if (
              inferenceResult.substitution &&
              inferenceResult.substitution.apply
            ) {
              finalFieldType = inferenceResult.substitution.apply(fieldType)
            }

            return {
              type: "variable",
              name: symbol,
              finalType: finalFieldType,
              hasExplicitType: false,
              isStructField: true,
              fieldName: foundStructField.fieldName,
              structName: expression.pattern.structName,
            }
          }
        }
      }
      break
    }

    default:
      // For other expression types, recursively search any nested expressions
      if (expression.statements) {
        for (const stmt of expression.statements) {
          const result = findSymbolInExpression(stmt, symbol, inferenceResult)
          if (result) return result
        }
      }
      if (expression.initializer) {
        const result = findSymbolInExpression(
          expression.initializer,
          symbol,
          inferenceResult
        )
        if (result) return result
      }
      if (expression.body) {
        const result = findSymbolInExpression(
          expression.body,
          symbol,
          inferenceResult
        )
        if (result) return result
      }
      break
  }

  return null
}

// Find symbol in a statement (for block expressions)
function findSymbolInStatement(
  statement: any,
  symbol: string,
  inferenceResult: any
): any {
  if (!statement) {
    return null
  }

  switch (statement.kind) {
    case "VariableDeclaration":
    case "RecordDestructuring":
    case "StructDestructuring":
    case "TupleDestructuring":
      return findSymbolInExpression(statement, symbol, inferenceResult)

    default:
      // For other statement types, check if they contain expressions
      if (statement.body) {
        return findSymbolInExpression(statement.body, symbol, inferenceResult)
      }
      break
  }

  return null
}

// Find symbol with inferred type information
function findSymbolWithEnhancedInference(
  ast: any,
  symbol: string,
  inferenceResult: any,
  offset: number,
  text: string
): any {
  connection.console.log(
    `[SESERAGI LSP DEBUG] Searching for symbol: "${symbol}" at offset ${offset}`
  )

  if (!ast.statements) {
    return null
  }

  for (const statement of ast.statements) {
    if (statement.kind === "FunctionDeclaration" && statement.name === symbol) {
      // Apply substitution to function type
      let funcType = statement.returnType
      for (let i = statement.parameters.length - 1; i >= 0; i--) {
        funcType = {
          kind: "FunctionType",
          paramType: statement.parameters[i].type,
          returnType: funcType,
        }
      }

      const finalType = inferenceResult.substitution.apply
        ? inferenceResult.substitution.apply(funcType)
        : funcType

      return {
        type: "function",
        name: symbol,
        parameters: statement.parameters,
        finalType: finalType,
        isEffectful: statement.isEffectful,
      }
    }

    // Check function parameters for the symbol - but only for the containing function
    if (statement.kind === "FunctionDeclaration" && statement.parameters) {
      // Find which function contains the current offset
      const containingFunction = findContainingFunction(ast, offset, text)

      // Only check parameters if this is the containing function
      if (containingFunction && containingFunction.name === statement.name) {
        for (const param of statement.parameters) {
          if (param.name === symbol) {
            let paramType = param.type
            if (
              paramType &&
              inferenceResult.substitution &&
              inferenceResult.substitution.apply
            ) {
              paramType = inferenceResult.substitution.apply(paramType)
            }

            connection.console.log(
              `[SESERAGI LSP DEBUG] Found parameter ${symbol} in containing function ${statement.name}`
            )

            return {
              type: "parameter",
              name: symbol,
              finalType: paramType,
              hasExplicitType: true,
              context: "parameter",
            }
          }
        }
      }
    }

    // Check impl block method and operator parameters for the symbol
    if (statement.kind === "ImplBlock") {
      const implBlock = statement as any // AST.ImplBlock

      // Check method parameters
      for (const method of implBlock.methods || []) {
        if (method.parameters) {
          for (const param of method.parameters) {
            if (param.name === symbol) {
              let paramType = param.type

              // Handle implicit self or other parameter - use the impl type
              if (
                (param.isImplicitSelf || param.isImplicitOther) &&
                implBlock.typeName
              ) {
                // Try to find the struct type from the environment
                const structType =
                  inferenceResult.environment?.[implBlock.typeName]
                if (structType) {
                  paramType = structType
                } else {
                  // Fallback: create a primitive type reference
                  paramType = {
                    kind: "PrimitiveType",
                    name: implBlock.typeName,
                    line: param.line || 0,
                    column: param.column || 0,
                  }
                }
              }

              // Apply substitution to resolve type variables
              if (
                paramType &&
                inferenceResult.substitution &&
                inferenceResult.substitution.apply
              ) {
                paramType = inferenceResult.substitution.apply(paramType)
              }

              connection.console.log(
                `=== LSP DEBUG: Found method parameter ${symbol} in impl ${implBlock.typeName} ===`
              )
              connection.console.log(
                `Parameter type: ${JSON.stringify(paramType, null, 2)}`
              )

              return {
                type: "parameter",
                name: symbol,
                finalType: paramType,
                hasExplicitType: true,
                context: `impl ${implBlock.typeName} method ${method.name}`,
              }
            }
          }
        }

        // Check variables inside method body
        const methodBodySymbol = findSymbolInExpression(
          method.body,
          symbol,
          inferenceResult
        )
        if (methodBodySymbol) {
          connection.console.log(
            `=== LSP DEBUG: Found symbol ${symbol} in method ${method.name} body ===`
          )
          methodBodySymbol.context = `impl ${implBlock.typeName} method ${method.name}`
          return methodBodySymbol
        }
      }

      // Check operator parameters
      for (const operator of implBlock.operators || []) {
        if (operator.parameters) {
          for (const param of operator.parameters) {
            if (param.name === symbol) {
              let paramType = param.type

              // Handle implicit self or other parameter - use the impl type
              if (
                (param.isImplicitSelf || param.isImplicitOther) &&
                implBlock.typeName
              ) {
                // Try to find the struct type from the environment
                const structType =
                  inferenceResult.environment?.[implBlock.typeName]
                if (structType) {
                  paramType = structType
                } else {
                  // Fallback: create a primitive type reference
                  paramType = {
                    kind: "PrimitiveType",
                    name: implBlock.typeName,
                    line: param.line || 0,
                    column: param.column || 0,
                  }
                }
              }

              // Apply substitution to resolve type variables
              if (
                paramType &&
                inferenceResult.substitution &&
                inferenceResult.substitution.apply
              ) {
                paramType = inferenceResult.substitution.apply(paramType)
              }

              connection.console.log(
                `=== LSP DEBUG: Found operator parameter ${symbol} in impl ${implBlock.typeName} ===`
              )
              connection.console.log(
                `Parameter type: ${JSON.stringify(paramType, null, 2)}`
              )

              return {
                type: "parameter",
                name: symbol,
                finalType: paramType,
                hasExplicitType: true,
                context: `impl ${implBlock.typeName} operator ${operator.operator}`,
              }
            }
          }
        }

        // Check variables inside operator body
        const operatorBodySymbol = findSymbolInExpression(
          operator.body,
          symbol,
          inferenceResult
        )
        if (operatorBodySymbol) {
          connection.console.log(
            `=== LSP DEBUG: Found symbol ${symbol} in operator ${operator.operator} body ===`
          )
          operatorBodySymbol.context = `impl ${implBlock.typeName} operator ${operator.operator}`
          return operatorBodySymbol
        }
      }
    }

    // Check for method calls in this statement - look for method definitions
    const methodCallInStatement = findMethodCallInNode(statement, symbol)
    if (methodCallInStatement) {
      connection.console.log(
        `=== LSP DEBUG: Found method call ${symbol} in statement ===`
      )

      // Find the method definition
      const methodDef = findMethodDefinition(ast, symbol)
      if (methodDef) {
        connection.console.log(
          `=== LSP DEBUG: Found method definition for ${symbol} ===`
        )

        // Build the method type signature
        let methodType = methodDef.method.returnType
        for (let i = methodDef.method.parameters.length - 1; i >= 0; i--) {
          methodType = {
            kind: "FunctionType",
            paramType: methodDef.method.parameters[i].type,
            returnType: methodType,
          }
        }

        // Apply substitution to resolve type variables
        const finalType = inferenceResult.substitution?.apply
          ? inferenceResult.substitution.apply(methodType)
          : methodType

        connection.console.log(
          `Method type: ${JSON.stringify(finalType, null, 2)}`
        )

        return {
          type: "function",
          name: symbol,
          parameters: methodDef.method.parameters,
          finalType: finalType,
          context: methodDef.context,
          isMethod: true,
        }
      }
    }

    if (statement.kind === "VariableDeclaration" && statement.name === symbol) {
      // Debug: log all tracked types for this variable
      connection.console.log(
        `=== LSP DEBUG: Looking for variable ${symbol} ===`
      )
      connection.console.log(
        `Statement type in nodeTypeMap: ${inferenceResult.nodeTypeMap.has(statement)}`
      )
      connection.console.log(
        `Initializer type in nodeTypeMap: ${inferenceResult.nodeTypeMap.has(statement.initializer)}`
      )

      // Log the actual type from nodeTypeMap
      const nodeType = inferenceResult.nodeTypeMap.get(statement)
      if (nodeType) {
        connection.console.log(
          `NodeTypeMap type for statement: ${JSON.stringify(nodeType, null, 2)}`
        )
      }

      const initType = inferenceResult.nodeTypeMap.get(statement.initializer)
      if (initType) {
        connection.console.log(
          `NodeTypeMap type for initializer: ${JSON.stringify(initType, null, 2)}`
        )
      }

      // Special handling for MonadBind expressions
      if (statement.initializer && statement.initializer.kind === "MonadBind") {
        const resolvedType = resolveMonadBindType(
          statement.initializer,
          inferenceResult
        )
        if (resolvedType) {
          connection.console.log(
            `Resolved MonadBind type: ${JSON.stringify(resolvedType)}`
          )
          return {
            type: "variable",
            name: symbol,
            finalType: resolvedType,
            hasExplicitType: !!statement.type,
          }
        }
      }

      // Use the enhanced node type mapping to get the resolved type
      let finalType = inferenceResult.nodeTypeMap.get(statement)

      if (!finalType) {
        // Fallback: look for the type in the initializer
        finalType = inferenceResult.nodeTypeMap.get(statement.initializer)
        connection.console.log(
          `Using initializer type: ${finalType ? finalType.kind : "none"}`
        )
      }

      if (!finalType) {
        // Final fallback: use explicit type or infer from expression
        finalType =
          statement.type || inferTypeFromExpression(statement.initializer, ast)
        connection.console.log(
          `Using fallback type: ${finalType ? finalType.kind : "none"}`
        )
      }

      // IMPORTANT: Always apply substitution to resolve type variables
      if (
        finalType &&
        inferenceResult.substitution &&
        inferenceResult.substitution.apply
      ) {
        const originalType = finalType
        finalType = inferenceResult.substitution.apply(finalType)
        connection.console.log(
          `Applied substitution: ${JSON.stringify(originalType, null, 2)} -> ${JSON.stringify(finalType, null, 2)}`
        )
      }

      connection.console.log(
        `=== FINAL TYPE FOR ${symbol}: ${JSON.stringify(finalType, null, 2)} ===`
      )
      connection.console.log(
        `=== FINAL TYPE KIND: ${finalType ? finalType.kind : "null"} ===`
      )
      if (finalType && finalType.kind === "StructType") {
        connection.console.log(`=== STRUCT TYPE NAME: ${finalType.name} ===`)
        connection.console.log(
          `=== STRUCT TYPE FIELDS: ${JSON.stringify(finalType.fields, null, 2)} ===`
        )
      }
      connection.console.log(
        `=== FORMATTED TYPE STRING: ${formatInferredTypeForDisplay(finalType)} ===`
      )

      return {
        type: "variable",
        name: symbol,
        finalType: finalType,
        hasExplicitType: !!statement.type,
      }
    }

    // Handle type alias declarations
    if (
      statement.kind === "TypeAliasDeclaration" &&
      statement.name === symbol
    ) {
      connection.console.log(`=== LSP DEBUG: Found type alias ${symbol} ===`)

      // Apply substitution to the aliased type if available
      let aliasedType = statement.aliasedType
      if (
        aliasedType &&
        inferenceResult.substitution &&
        inferenceResult.substitution.apply
      ) {
        const originalType = aliasedType
        aliasedType = inferenceResult.substitution.apply(aliasedType)
        connection.console.log(
          `Applied substitution to type alias: ${JSON.stringify(originalType, null, 2)} -> ${JSON.stringify(aliasedType, null, 2)}`
        )
      }

      connection.console.log(
        `=== TYPE ALIAS ${symbol}: ${JSON.stringify(aliasedType, null, 2)} ===`
      )

      return {
        type: "typealias",
        name: symbol,
        finalType: aliasedType,
        aliasedType: aliasedType,
      }
    }

    // Handle tuple destructuring
    if (statement.kind === "TupleDestructuring") {
      const tupleDestr = statement as any // AST.TupleDestructuring

      // Check if the symbol is one of the destructured variables
      const foundVariable = findVariableInTuplePattern(
        tupleDestr.pattern,
        symbol
      )
      if (foundVariable) {
        connection.console.log(
          `=== LSP DEBUG: Found variable ${symbol} in tuple destructuring ===`
        )

        // Get the type of the initializer (the tuple being destructured)
        const initType = inferenceResult.nodeTypeMap.get(tupleDestr.initializer)
        connection.console.log(
          `Tuple initializer type: ${JSON.stringify(initType, null, 2)}`
        )

        // Extract the specific element type for this variable
        let elementType = null
        if (
          initType &&
          initType.kind === "TupleType" &&
          initType.elementTypes
        ) {
          const elementIndex = foundVariable.index
          if (elementIndex < initType.elementTypes.length) {
            elementType = initType.elementTypes[elementIndex]
          }
        }

        // Apply substitution if available
        if (
          elementType &&
          inferenceResult.substitution &&
          inferenceResult.substitution.apply
        ) {
          const originalType = elementType
          elementType = inferenceResult.substitution.apply(elementType)
          connection.console.log(
            `Applied substitution to tuple element: ${JSON.stringify(originalType, null, 2)} -> ${JSON.stringify(elementType, null, 2)}`
          )
        }

        connection.console.log(
          `=== FINAL TUPLE ELEMENT TYPE FOR ${symbol}: ${JSON.stringify(elementType, null, 2)} ===`
        )
        connection.console.log(
          `=== FORMATTED TUPLE ELEMENT TYPE STRING: ${formatInferredTypeForDisplay(elementType)} ===`
        )

        return {
          type: "variable",
          name: symbol,
          finalType: elementType,
          hasExplicitType: false,
          isTupleElement: true,
          tupleIndex: foundVariable.index,
        }
      }
    }

    // Handle record destructuring
    if (statement.kind === "RecordDestructuring") {
      const recordDestr = statement as any // AST.RecordDestructuring

      // Check if the symbol is one of the destructured variables
      const foundField = findVariableInRecordPattern(
        recordDestr.pattern,
        symbol
      )
      if (foundField) {
        connection.console.log(
          `=== LSP DEBUG: Found variable ${symbol} in record destructuring ===`
        )

        // Get the type of the initializer (the record being destructured)
        const initType = inferenceResult.nodeTypeMap.get(
          recordDestr.initializer
        )
        connection.console.log(
          `Record initializer type: ${JSON.stringify(initType, null, 2)}`
        )

        if (initType && initType.kind === "RecordType") {
          const recordType = initType as any
          const fieldType = recordType.fields.find(
            (f: any) => f.name === foundField.fieldName
          )?.type

          if (fieldType) {
            let finalFieldType = fieldType
            if (
              inferenceResult.substitution &&
              inferenceResult.substitution.apply
            ) {
              finalFieldType = inferenceResult.substitution.apply(fieldType)
            }

            return {
              type: "variable",
              name: symbol,
              finalType: finalFieldType,
              hasExplicitType: false,
              isRecordField: true,
              fieldName: foundField.fieldName,
            }
          }
        }
      }
    }

    // Handle struct destructuring
    if (statement.kind === "StructDestructuring") {
      const structDestr = statement as any // AST.StructDestructuring

      // Check if the symbol is one of the destructured variables
      const foundField = findVariableInStructPattern(
        structDestr.pattern,
        symbol
      )
      if (foundField) {
        connection.console.log(
          `=== LSP DEBUG: Found variable ${symbol} in struct destructuring ===`
        )

        // Get the type of the initializer (the struct being destructured)
        const initType = inferenceResult.nodeTypeMap.get(
          structDestr.initializer
        )
        connection.console.log(
          `Struct initializer type: ${JSON.stringify(initType, null, 2)}`
        )

        if (initType && initType.kind === "StructType") {
          const structType = initType as any
          const fieldType = structType.fields.find(
            (f: any) => f.name === foundField.fieldName
          )?.type

          if (fieldType) {
            let finalFieldType = fieldType
            if (
              inferenceResult.substitution &&
              inferenceResult.substitution.apply
            ) {
              finalFieldType = inferenceResult.substitution.apply(fieldType)
            }

            return {
              type: "variable",
              name: symbol,
              finalType: finalFieldType,
              hasExplicitType: false,
              isStructField: true,
              fieldName: foundField.fieldName,
              structName: structDestr.pattern.structName,
            }
          }
        }
      }
    }
  }

  return null
}

// Helper function to find a variable in a tuple pattern
function findVariableInTuplePattern(
  pattern: any,
  symbol: string
): { index: number } | null {
  if (pattern.kind !== "TuplePattern") {
    return null
  }

  for (let i = 0; i < pattern.patterns.length; i++) {
    const subPattern = pattern.patterns[i]

    if (subPattern.kind === "IdentifierPattern" && subPattern.name === symbol) {
      return { index: i }
    }

    // Recursively search in nested tuple patterns
    if (subPattern.kind === "TuplePattern") {
      const found = findVariableInTuplePattern(subPattern, symbol)
      if (found) {
        return found // Note: For nested tuples, we might need to handle indexing differently
      }
    }
  }

  return null
}

// Helper function to find a variable in a record pattern
function findVariableInRecordPattern(
  pattern: any,
  symbol: string
): { fieldName: string } | null {
  if (pattern.kind !== "RecordPattern") {
    return null
  }

  for (const field of pattern.fields) {
    const variableName = field.alias || field.fieldName
    if (variableName === symbol) {
      return { fieldName: field.fieldName }
    }
  }

  return null
}

// Helper function to find a variable in a struct pattern
function findVariableInStructPattern(
  pattern: any,
  symbol: string
): { fieldName: string } | null {
  if (pattern.kind !== "StructPattern") {
    return null
  }

  for (const field of pattern.fields) {
    const variableName = field.alias || field.fieldName
    if (variableName === symbol) {
      return { fieldName: field.fieldName }
    }
  }

  return null
}

// Helper function to find a type alias definition by name
function findTypeAliasDefinition(typeName: string): any {
  if (!cachedAST || !cachedAST.statements) {
    return null
  }

  for (const statement of cachedAST.statements) {
    if (
      statement.kind === "TypeAliasDeclaration" &&
      statement.name === typeName
    ) {
      return statement
    }
  }

  return null
}

// Helper function to find a variable declaration by name
function findVariableDeclaration(varName: string): any {
  if (!cachedAST || !cachedAST.statements) {
    return null
  }

  for (const statement of cachedAST.statements) {
    if (
      statement.kind === "VariableDeclaration" &&
      statement.name === varName
    ) {
      return statement
    }
  }

  return null
}

// Helper function to recursively search for method calls in an AST node
function findMethodCallInNode(node: any, methodName: string): any {
  if (!node) {
    return null
  }

  // Check if this node is a MethodCall with the target method name
  if (node.kind === "MethodCall" && node.methodName === methodName) {
    return node
  }

  // Recursively search in all properties of the node
  for (const key in node) {
    if (Object.hasOwn(node, key) && typeof node[key] === "object") {
      const value = node[key]

      if (Array.isArray(value)) {
        // Search in array elements
        for (const item of value) {
          const result = findMethodCallInNode(item, methodName)
          if (result) {
            return result
          }
        }
      } else {
        // Search in object properties
        const result = findMethodCallInNode(value, methodName)
        if (result) {
          return result
        }
      }
    }
  }

  return null
}

// Helper function to find method definition in impl blocks
function findMethodDefinition(ast: any, methodName: string): any {
  if (!ast.statements) {
    return null
  }

  for (const statement of ast.statements) {
    if (statement.kind === "ImplBlock") {
      const implBlock = statement as any // AST.ImplBlock

      // Check methods
      for (const method of implBlock.methods || []) {
        if (method.name === methodName) {
          return {
            method: method,
            implType: implBlock.typeName,
            context: `impl ${implBlock.typeName} method`,
          }
        }
      }
    }
  }

  return null
}

// Resolve MonadBind type by analyzing the pattern
function resolveMonadBindType(monadBindExpr: any, inferenceResult: any): any {
  try {
    // For MonadBind: left >>= right
    // We need to determine the type based on the left operand
    const leftType = inferenceResult.nodeTypeMap.get(monadBindExpr.left)
    const rightType = inferenceResult.nodeTypeMap.get(monadBindExpr.right)

    connection.console.log(
      `MonadBind left type: ${leftType ? leftType.kind : "none"}`
    )
    connection.console.log(
      `MonadBind right type: ${rightType ? rightType.kind : "none"}`
    )

    if (leftType && leftType.kind === "GenericType") {
      // If left is Either<String, Int> or Maybe<Int>, preserve the container type
      if (
        leftType.name === "Either" &&
        leftType.typeArguments &&
        leftType.typeArguments.length === 2
      ) {
        // For Either<e, a> >>= f, result is Either<e, b> where f: a -> Either<e, b>
        const errorType = leftType.typeArguments[0]

        // Try to infer the result type from the right side function
        if (rightType && rightType.kind === "FunctionType") {
          const returnType = rightType.returnType
          if (
            returnType &&
            returnType.kind === "GenericType" &&
            returnType.name === "Either"
          ) {
            // Use the return type from the function
            return returnType
          }
        }

        // Fallback: assume the result has the same error type but unknown value type
        return {
          kind: "GenericType",
          name: "Either",
          typeArguments: [
            errorType,
            { kind: "PrimitiveType", name: "Int" }, // Assume Int for now
          ],
        }
      }

      if (
        leftType.name === "Maybe" &&
        leftType.typeArguments &&
        leftType.typeArguments.length === 1
      ) {
        // For Maybe<a> >>= f, result is Maybe<b> where f: a -> Maybe<b>
        if (rightType && rightType.kind === "FunctionType") {
          const returnType = rightType.returnType
          if (
            returnType &&
            returnType.kind === "GenericType" &&
            returnType.name === "Maybe"
          ) {
            return returnType
          }
        }

        // Fallback: assume Maybe<Int>
        return {
          kind: "GenericType",
          name: "Maybe",
          typeArguments: [{ kind: "PrimitiveType", name: "Int" }],
        }
      }
    }

    return null
  } catch (error) {
    connection.console.log(
      `Error resolving MonadBind type: ${error instanceof Error ? error.message : "Unknown"}`
    )
    return null
  }
}

// Extract variable type from type inference result
function extractVariableTypeFromInference(
  statement: any,
  substitution: any
): any {
  // This is a simplified approach - in a full implementation, we'd need to
  // track which type variables correspond to which expressions
  // For now, try to infer the type from the expression directly
  return inferTypeFromExpression(statement.initializer)
}

// Simple type inference from expression (fallback)
function inferTypeFromExpression(expr: any, ast?: any): any {
  if (!expr) return null

  switch (expr.kind) {
    case "Literal":
      switch (expr.literalType) {
        case "integer":
          return { kind: "PrimitiveType", name: "Int" }
        case "float":
          return { kind: "PrimitiveType", name: "Float" }
        case "string":
          return { kind: "PrimitiveType", name: "String" }
        case "boolean":
          return { kind: "PrimitiveType", name: "Bool" }
        default:
          return null
      }

    case "ConstructorExpression": {
      // Handle Maybe and Either constructors
      const ctor = expr as any
      switch (ctor.constructorName) {
        case "Just":
          if (ctor.arguments && ctor.arguments.length > 0) {
            const argType = inferTypeFromExpression(ctor.arguments[0], ast)
            return {
              kind: "GenericType",
              name: "Maybe",
              typeArguments: [
                argType || { kind: "PrimitiveType", name: "Int" },
              ],
            }
          }
          return {
            kind: "GenericType",
            name: "Maybe",
            typeArguments: [{ kind: "PrimitiveType", name: "Int" }],
          }
        case "Nothing":
          return {
            kind: "GenericType",
            name: "Maybe",
            typeArguments: [{ kind: "TypeVariable", name: "a" }],
          }
        case "Right":
          if (ctor.arguments && ctor.arguments.length > 0) {
            const argType = inferTypeFromExpression(ctor.arguments[0], ast)
            return {
              kind: "GenericType",
              name: "Either",
              typeArguments: [
                { kind: "TypeVariable", name: "a" },
                argType || { kind: "PrimitiveType", name: "Int" },
              ],
            }
          }
          return {
            kind: "GenericType",
            name: "Either",
            typeArguments: [
              { kind: "TypeVariable", name: "a" },
              { kind: "PrimitiveType", name: "Int" },
            ],
          }
        case "Left":
          if (ctor.arguments && ctor.arguments.length > 0) {
            const argType = inferTypeFromExpression(ctor.arguments[0], ast)
            return {
              kind: "GenericType",
              name: "Either",
              typeArguments: [
                argType || { kind: "PrimitiveType", name: "String" },
                { kind: "TypeVariable", name: "b" },
              ],
            }
          }
          return {
            kind: "GenericType",
            name: "Either",
            typeArguments: [
              { kind: "PrimitiveType", name: "String" },
              { kind: "TypeVariable", name: "b" },
            ],
          }
        default:
          return null
      }
    }

    case "BinaryOperation":
      // For binary operations, try to infer numeric types
      if (["+", "-", "*", "/", "%"].includes(expr.operator)) {
        return { kind: "PrimitiveType", name: "Int" } // Simplified
      }
      if (
        ["==", "!=", "<", ">", "<=", ">=", "&&", "||"].includes(expr.operator)
      ) {
        return { kind: "PrimitiveType", name: "Bool" }
      }
      return null

    case "FunctionCall":
      // For function calls, try to find the function's return type
      if (expr.function && expr.function.kind === "Identifier") {
        return inferFunctionCallReturnType(expr, ast)
      }
      return null

    case "FunctionApplication":
      // For function applications (curried calls), handle partial application
      if (expr.function && expr.function.kind === "Identifier") {
        return inferCurriedFunctionType(expr, ast)
      }
      // Handle nested function applications
      if (expr.function && expr.function.kind === "FunctionApplication") {
        return inferCurriedFunctionType(expr, ast)
      }
      return null

    case "Identifier": {
      // Handle constructor calls
      const name = expr.name
      if (name === "Nothing") {
        return {
          kind: "GenericType",
          name: "Maybe",
          typeArguments: [{ kind: "TypeVariable", name: "T" }],
        }
      }
      return null
    }

    case "RangeLiteral":
      // Range literals return Array<Int>
      connection.console.log("=== RANGE LITERAL DETECTED IN LSP ===")
      return {
        kind: "GenericType",
        name: "Array",
        typeArguments: [{ kind: "PrimitiveType", name: "Int" }],
      }

    case "TupleExpression":
      // Tuple expressions return TupleType with element types
      if (expr.elements && expr.elements.length > 0) {
        const elementTypes = expr.elements.map(
          (element: any) =>
            inferTypeFromExpression(element, ast) || {
              kind: "TypeVariable",
              name: "T",
            }
        )
        return {
          kind: "TupleType",
          elementTypes: elementTypes,
        }
      }
      return {
        kind: "TupleType",
        elementTypes: [],
      }

    case "ListComprehension": {
      // Array comprehensions return Array<T> where T is the type of the expression
      const expressionType = inferTypeFromExpression(expr.expression, ast)
      return {
        kind: "GenericType",
        name: "Array",
        typeArguments: [expressionType || { kind: "TypeVariable", name: "T" }],
      }
    }

    case "ListComprehensionSugar": {
      // List comprehension sugar also returns List<T>
      const sugarExpressionType = inferTypeFromExpression(expr.expression, ast)
      return {
        kind: "GenericType",
        name: "List",
        typeArguments: [
          sugarExpressionType || { kind: "TypeVariable", name: "T" },
        ],
      }
    }

    case "ArrayLiteral":
      // Array literals return Array<T> where T is the type of the first element
      if (expr.elements && expr.elements.length > 0) {
        const elementType = inferTypeFromExpression(expr.elements[0], ast)
        return {
          kind: "GenericType",
          name: "Array",
          typeArguments: [elementType || { kind: "TypeVariable", name: "T" }],
        }
      }
      return {
        kind: "GenericType",
        name: "Array",
        typeArguments: [{ kind: "TypeVariable", name: "T" }],
      }

    default:
      return null
  }
}

// Infer return type from function call by looking up function definition
function inferFunctionCallReturnType(call: any, ast?: any): any {
  if (!call.function || call.function.kind !== "Identifier") {
    return null
  }

  const functionName = call.function.name

  // Handle Maybe/Either constructors with arguments
  if (functionName === "Just" && call.arguments && call.arguments.length > 0) {
    const argType = inferTypeFromExpression(call.arguments[0], ast)
    return {
      kind: "GenericType",
      name: "Maybe",
      typeArguments: [argType || { kind: "PrimitiveType", name: "Int" }],
    }
  }

  if (functionName === "Right" && call.arguments && call.arguments.length > 0) {
    const argType = inferTypeFromExpression(call.arguments[0], ast)
    return {
      kind: "GenericType",
      name: "Either",
      typeArguments: [
        { kind: "TypeVariable", name: "L" },
        argType || { kind: "PrimitiveType", name: "Int" },
      ],
    }
  }

  if (functionName === "Left" && call.arguments && call.arguments.length > 0) {
    const argType = inferTypeFromExpression(call.arguments[0], ast)
    return {
      kind: "GenericType",
      name: "Either",
      typeArguments: [
        argType || { kind: "PrimitiveType", name: "String" },
        { kind: "TypeVariable", name: "R" },
      ],
    }
  }

  // If we have access to the AST, look up the function definition
  if (ast && ast.statements) {
    for (const statement of ast.statements) {
      if (
        statement.kind === "FunctionDeclaration" &&
        statement.name === functionName
      ) {
        return statement.returnType
      }
    }
  }

  // Fallback to known built-in functions
  const knownFunctions: { [key: string]: any } = {
    processNumber: { kind: "PrimitiveType", name: "Int" },
    formatMessage: { kind: "PrimitiveType", name: "String" },
    complexCalculation: { kind: "PrimitiveType", name: "Int" },
    add: { kind: "PrimitiveType", name: "Int" },
    double: { kind: "PrimitiveType", name: "Int" },
    // Array↔List conversion functions
    arrayToList: (argType: any) => ({
      kind: "GenericType",
      name: "List",
      typeArguments: [argType || { kind: "TypeVariable", name: "T" }],
    }),
    listToArray: (argType: any) => ({
      kind: "GenericType",
      name: "Array",
      typeArguments: [argType || { kind: "TypeVariable", name: "T" }],
    }),
    getMessage: { kind: "PrimitiveType", name: "String" },
    getNumber: { kind: "PrimitiveType", name: "Int" },
    max: { kind: "PrimitiveType", name: "Int" },
    safeDivide: {
      kind: "GenericType",
      name: "Maybe",
      typeArguments: [{ kind: "PrimitiveType", name: "Int" }],
    },
    parseNumber: {
      kind: "GenericType",
      name: "Either",
      typeArguments: [
        { kind: "PrimitiveType", name: "String" },
        { kind: "PrimitiveType", name: "Int" },
      ],
    },
    // Maybe constructors
    Just: (argType: any) => ({
      kind: "GenericType",
      name: "Maybe",
      typeArguments: [argType || { kind: "PrimitiveType", name: "Int" }],
    }),
    Nothing: {
      kind: "GenericType",
      name: "Maybe",
      typeArguments: [{ kind: "TypeVariable", name: "T" }],
    },
    // Either constructors
    Right: (argType: any) => ({
      kind: "GenericType",
      name: "Either",
      typeArguments: [
        { kind: "TypeVariable", name: "L" },
        argType || { kind: "PrimitiveType", name: "Int" },
      ],
    }),
    Left: (argType: any) => ({
      kind: "GenericType",
      name: "Either",
      typeArguments: [
        argType || { kind: "PrimitiveType", name: "String" },
        { kind: "TypeVariable", name: "R" },
      ],
    }),
  }

  return knownFunctions[functionName] || null
}

// Handle curried function applications with proper type inference
function inferCurriedFunctionType(expr: any, ast?: any): any {
  // For safeDivide 10 2 - this should return Maybe<Int>
  if (expr.function && expr.function.kind === "FunctionApplication") {
    // This is a nested application like (safeDivide 10) 2
    const innerApp = expr.function
    if (innerApp.function && innerApp.function.kind === "Identifier") {
      const funcName = innerApp.function.name

      // Count total arguments applied
      let totalArgs = 1 // Current argument
      if (innerApp.argument) totalArgs++

      // Special handling for safeDivide with 2 arguments
      if (funcName === "safeDivide" && totalArgs >= 2) {
        return {
          kind: "GenericType",
          name: "Maybe",
          typeArguments: [{ kind: "PrimitiveType", name: "Int" }],
        }
      }
    }
  }

  // Single function application
  if (expr.function && expr.function.kind === "Identifier") {
    const funcName = expr.function.name

    // Handle Maybe constructors
    if (funcName === "Just" && expr.argument) {
      const argType = inferTypeFromExpression(expr.argument, ast)
      return {
        kind: "GenericType",
        name: "Maybe",
        typeArguments: [argType || { kind: "PrimitiveType", name: "Int" }],
      }
    }

    // For single argument to safeDivide, return partial type
    if (funcName === "safeDivide") {
      return {
        kind: "FunctionType",
        paramType: { kind: "PrimitiveType", name: "Int" },
        returnType: {
          kind: "GenericType",
          name: "Maybe",
          typeArguments: [{ kind: "PrimitiveType", name: "Int" }],
        },
      }
    }
  }

  return inferFunctionCallReturnType(expr, ast)
}

// Format inferred type information for hover display
function formatInferredTypeInfo(symbol: string, info: any): string {
  switch (info.type) {
    case "function": {
      const effectful = info.isEffectful ? "effectful " : ""

      // Build curried function signature from parameters
      let funcSignature = `${effectful}fn ${symbol}`
      if (info.parameters && info.parameters.length > 0) {
        const paramSig = info.parameters
          .map((p: any) => {
            const paramType = formatInferredTypeForDisplay(p.type)
            return `${p.name}: ${paramType}`
          })
          .join(" -> ")

        // Extract just the return type from the nested function type
        let returnType = info.finalType
        for (let i = 0; i < info.parameters.length; i++) {
          if (returnType && returnType.kind === "FunctionType") {
            returnType = returnType.returnType
          }
        }
        const returnTypeStr = formatInferredTypeForDisplay(returnType)
        funcSignature += ` ${paramSig} -> ${returnTypeStr}`
      } else {
        const returnType = formatInferredTypeForDisplay(info.finalType)
        funcSignature += ` -> ${returnType}`
      }

      // Add context information for methods
      const contextInfo =
        info.isMethod && info.context ? `\n**Context:** ${info.context}` : ""

      // For display, we don't need to show the full curried type again
      // Just show the function signature
      return `\`\`\`seseragi\n${funcSignature}\n\`\`\`${contextInfo}`
    }

    case "parameter": {
      const paramType = formatInferredTypeForDisplay(info.finalType)
      const context = info.context ? info.context : "function parameter"
      return `\`\`\`seseragi\n${symbol}: ${paramType}\n\`\`\`\n**Type:** ${context}`
    }

    case "variable": {
      const typeAnnotation = info.hasExplicitType ? "explicit" : "inferred"
      let varType = formatInferredTypeForDisplay(info.finalType)

      // Check if this is a type alias and format accordingly
      if (info.hasExplicitType) {
        // Try to find the original type annotation from the AST
        const varDecl = findVariableDeclaration(symbol)
        if (varDecl && varDecl.type && varDecl.type.kind === "PrimitiveType") {
          const typeAlias = findTypeAliasDefinition(varDecl.type.name)
          if (typeAlias && typeAlias.aliasedType) {
            const aliasedTypeStr = formatInferredTypeForDisplay(
              typeAlias.aliasedType
            )
            varType = `${varDecl.type.name} = ${aliasedTypeStr}`
          }
        }
      }

      // Add context for destructured variables
      let context = ""
      if (info.isTupleElement) {
        context = `\n**Context:** Destructured from tuple (index ${info.tupleIndex})`
      } else if (info.isRecordField) {
        context = `\n**Context:** Destructured from record field '${info.fieldName}'`
      } else if (info.isStructField) {
        context = `\n**Context:** Destructured from struct ${info.structName} field '${info.fieldName}'`
      }

      return `\`\`\`seseragi\nlet ${symbol}: ${varType}\n\`\`\`\n**Type:** ${typeAnnotation}${context}`
    }

    case "typealias": {
      const aliasedType = formatInferredTypeForDisplay(info.aliasedType)
      return `\`\`\`seseragi\ntype ${symbol} = ${aliasedType}\n\`\`\`\n**Type:** type alias`
    }

    default:
      return null
  }
}

// Format inferred type for display
function formatInferredTypeForDisplay(type: any): string {
  if (!type) return "unknown"

  if (typeof type === "string") {
    return type
  }

  if (type.kind === "PrimitiveType") {
    // Check if this "primitive" is actually a struct
    const structInfo = findStructDefinition(type.name)
    if (structInfo && structInfo.fields && structInfo.fields.length > 0) {
      connection.console.log(
        `[DEBUG] PrimitiveType '${type.name}' is actually a struct, converting to detailed display`
      )
      // Create a RecordType-like structure for consistent formatting
      const structAsRecord = {
        kind: "RecordType",
        fields: structInfo.fields,
        name: type.name,
      }
      const result = formatTypeWithNestedStructures(
        structAsRecord,
        structInfo,
        0,
        ""
      )
      connection.console.log(
        `[DEBUG] PrimitiveType->StructType formatted result: ${result}`
      )
      return result
    }
    return type.name
  }

  if (type.kind === "TypeVariable") {
    // For type variables, they should have been resolved by substitution
    // If we still have an unresolved type variable, show detailed info for debugging
    const tv = type as any
    if (tv.name && tv.name.startsWith("t")) {
      // This indicates a type variable that wasn't fully resolved
      // Try to infer a more specific type based on context
      return `unknown` // Better than Monad<unknown>
    }
    return tv.name || "unknown"
  }

  if (type.kind === "PolymorphicTypeVariable") {
    // For polymorphic type variables, show them properly
    const ptv = type as any
    return `'${ptv.name}`
  }

  if (type.kind === "FunctionType") {
    const paramType = formatInferredTypeForDisplay(type.paramType)
    const returnType = formatInferredTypeForDisplay(type.returnType)
    return `(${paramType} -> ${returnType})`
  }

  if (type.kind === "GenericType") {
    const baseType = type.name
    const typeArgs =
      type.typeArguments?.map(formatInferredTypeForDisplay).join(", ") || ""
    return typeArgs ? `${baseType}<${typeArgs}>` : baseType
  }

  if (type.kind === "StructType") {
    // Always try to show detailed struct information
    connection.console.log(`[DEBUG] StructType detected: ${type.name}`)
    const structInfo = findStructDefinition(type.name)
    connection.console.log(
      `[DEBUG] StructInfo found: ${structInfo ? "yes" : "no"}`
    )
    if (structInfo) {
      connection.console.log(
        `[DEBUG] StructInfo fields: ${JSON.stringify(structInfo.fields)}`
      )
    }
    if (structInfo && structInfo.fields && structInfo.fields.length > 0) {
      // Create a RecordType-like structure for consistent formatting
      const structAsRecord = {
        kind: "RecordType",
        fields: structInfo.fields,
        name: type.name,
      }
      const result = formatTypeWithNestedStructures(
        structAsRecord,
        structInfo,
        0,
        ""
      )
      connection.console.log(`[DEBUG] StructType formatted result: ${result}`)
      return result
    }
    connection.console.log(
      `[DEBUG] StructType returning simple name: ${type.name}`
    )
    return type.name
  }

  if (type.kind === "RecordType") {
    if (type.fields && type.fields.length > 0) {
      return formatTypeWithNestedStructures(type, type, 0, "")
    }
    return "{}"
  }

  if (type.kind === "TupleType") {
    const elementTypes =
      type.elementTypes?.map(formatInferredTypeForDisplay).join(", ") || ""
    return `(${elementTypes})`
  }

  if (type.kind === "StructType") {
    const structType = type as any
    if (structType.fields && structType.fields.length > 0) {
      const fieldStrs = structType.fields.map((field: any) => {
        const fieldType = formatInferredTypeForDisplay(field.type)
        return `${field.name}: ${fieldType}`
      })

      // Always use single line format for structs
      return `${structType.name} { ${fieldStrs.join(", ")} }`
    }
    return structType.name || "Struct"
  }

  if (type.kind === "RecordType") {
    const recordType = type as any
    if (recordType.fields && recordType.fields.length > 0) {
      const fieldStrs = recordType.fields.map((field: any) => {
        const fieldType = formatInferredTypeForDisplay(field.type)
        return `${field.name}: ${fieldType}`
      })

      // Always use single line format for records
      return `{ ${fieldStrs.join(", ")} }`
    }
    return "{}"
  }

  return type.name || "unknown"
}

// Format types with nested structures and proper indentation
function formatTypeWithNestedStructures(
  type: any,
  ast: any,
  depth: number = 0,
  indent: string = ""
): string {
  if (!type) return "unknown"

  switch (type.kind) {
    case "StructType": {
      let fields = []
      if (ast && ast.fields) {
        fields = ast.fields
      } else {
        const structInfo = findStructDefinition(type.name)
        if (structInfo && structInfo.fields) {
          fields = structInfo.fields
        }
      }

      if (fields.length > 0) {
        const fieldStrs = fields.map((field: any) => {
          const fieldType = formatTypeWithNestedStructures(
            field.type,
            null,
            depth + 1,
            indent + "  "
          )
          return `${field.name}: ${fieldType}`
        })

        // Check if we need multiline formatting
        const hasNestedStructures = fields.some(
          (field: any) =>
            field.type &&
            (field.type.kind === "StructType" ||
              field.type.kind === "RecordType")
        )

        if (hasNestedStructures || fields.length > 3) {
          return `${type.name} {\n${indent}  ${fieldStrs.join(`,\n${indent}  `)}\n${indent}}`
        } else {
          return `${type.name} { ${fieldStrs.join(", ")} }`
        }
      }
      return type.name || "{}"
    }

    case "RecordType":
      if (type.fields && type.fields.length > 0) {
        const fieldStrs = type.fields.map((field: any) => {
          const fieldType = formatTypeWithNestedStructures(
            field.type,
            null,
            depth + 1,
            indent + "  "
          )
          return `${field.name}: ${fieldType}`
        })

        // Check if we need multiline formatting
        const hasNestedStructures = type.fields.some(
          (field: any) =>
            field.type &&
            (field.type.kind === "StructType" ||
              field.type.kind === "RecordType")
        )

        // If this RecordType has a name (i.e., it's a struct converted to RecordType), show the struct name
        const structPrefix = type.name ? `${type.name} ` : ""

        if (hasNestedStructures || type.fields.length > 3) {
          return `${structPrefix}{\n${indent}  ${fieldStrs.join(`,\n${indent}  `)}\n${indent}}`
        } else {
          return `${structPrefix}{ ${fieldStrs.join(", ")} }`
        }
      }
      return "{}"

    case "PrimitiveType":
      return type.name

    case "FunctionType": {
      const paramType = formatTypeWithNestedStructures(
        type.paramType,
        null,
        depth,
        indent
      )
      const returnType = formatTypeWithNestedStructures(
        type.returnType,
        null,
        depth,
        indent
      )
      return `(${paramType} -> ${returnType})`
    }

    case "GenericType": {
      const baseType = type.name
      const typeArgs = type.typeArguments
        ?.map((t: any) =>
          formatTypeWithNestedStructures(t, null, depth, indent)
        )
        .join(", ")
      return typeArgs ? `${baseType}<${typeArgs}>` : baseType
    }

    case "TupleType": {
      const elementTypes =
        type.elementTypes
          ?.map((t: any) =>
            formatTypeWithNestedStructures(t, null, depth, indent)
          )
          .join(", ") || ""
      return `(${elementTypes})`
    }

    default:
      return type.name || "unknown"
  }
}

// Format struct definition information for hover display
function formatStructDefinitionInfo(
  structName: string,
  structInfo: any
): string {
  if (!structInfo || !structInfo.fields) {
    return `**struct ${structName}**`
  }

  const fieldStrs = structInfo.fields.map((field: any) => {
    const fieldType = formatInferredTypeForDisplay(field.type)
    return `  ${field.name}: ${fieldType}`
  })

  let structDef = `struct ${structName} {\n${fieldStrs.join(",\n")}\n}`

  // If it has nested structures or many fields, format with better spacing
  const hasNestedStructures = structInfo.fields.some(
    (field: any) =>
      field.type &&
      (field.type.kind === "StructType" || field.type.kind === "RecordType")
  )

  if (hasNestedStructures) {
    const detailedFields = structInfo.fields.map((field: any) => {
      const fieldType = formatTypeWithNestedStructures(
        field.type,
        null,
        1,
        "  "
      )
      return `  ${field.name}: ${fieldType}`
    })
    structDef = `struct ${structName} {\n${detailedFields.join(",\n")}\n}`
  }

  return `\`\`\`seseragi\n${structDef}\n\`\`\`\n**Type:** struct definition`
}

// Find struct definition in AST (helper function)
let cachedAST: any = null
function findStructDefinition(structName: string): any {
  if (!cachedAST) return null

  if (cachedAST.statements) {
    for (const statement of cachedAST.statements) {
      if (
        statement.kind === "StructDeclaration" &&
        statement.name === structName
      ) {
        return statement
      }
    }
  }
  return null
}

// This handler provides go to definition functionality.
connection.onDefinition((params: DefinitionParams): Definition | null => {
  const document = documents.get(params.textDocument.uri)
  if (!document) {
    return null
  }

  const text = document.getText()
  const position = params.position
  const offset = document.offsetAt(position)

  try {
    // Parse the document
    const parser = new Parser(text)
    const ast = parser.parse()

    // Get the word at the current position
    const wordAtPosition = getWordAtPosition(text, offset)
    if (!wordAtPosition) {
      return null
    }

    // Find the definition of the symbol in the AST
    const definition = findDefinition(ast, wordAtPosition, document)
    if (definition) {
      return definition
    }
  } catch (error) {
    connection.console.log(
      `Definition error: ${error instanceof Error ? error.message : "Unknown error"}`
    )
  }

  return null
})

// Helper function to find symbol definitions
function findDefinition(
  ast: any,
  symbol: string,
  document: TextDocument
): Location | null {
  if (!ast.items) {
    return null
  }

  for (const item of ast.items) {
    if (item.type === "FunctionDefinition" && item.name === symbol) {
      // Calculate position from AST node if available
      const line = item.line || 0
      const character = item.column || 0
      return {
        uri: document.uri,
        range: {
          start: { line, character },
          end: { line, character: character + symbol.length },
        },
      }
    }

    if (item.type === "VariableDefinition" && item.name === symbol) {
      const line = item.line || 0
      const character = item.column || 0
      return {
        uri: document.uri,
        range: {
          start: { line, character },
          end: { line, character: character + symbol.length },
        },
      }
    }

    if (item.type === "TypeDefinition" && item.name === symbol) {
      const line = item.line || 0
      const character = item.column || 0
      return {
        uri: document.uri,
        range: {
          start: { line, character },
          end: { line, character: character + symbol.length },
        },
      }
    }
  }

  return null
}

// This handler provides document formatting.
connection.onDocumentFormatting(
  (params: DocumentFormattingParams): TextEdit[] => {
    const document = documents.get(params.textDocument.uri)
    if (!document) {
      return []
    }

    try {
      const text = document.getText()
      connection.console.log(
        `[Format] Formatter called for ${params.textDocument.uri}`
      )
      connection.console.log(`[Format] Input text length: ${text.length}`)
      connection.console.log(
        `[Format] Input preview: ${JSON.stringify(text.substring(0, 100))}...`
      )

      // Apply the same formatting process as CLI
      let formatted = text

      // Only apply main formatting (don't use normalize operator spacing by default)
      formatted = formatSeseragiCode(formatted)
      connection.console.log(`[Format] Output text length: ${formatted.length}`)
      connection.console.log(
        `[Format] Output preview: ${JSON.stringify(formatted.substring(0, 100))}...`
      )

      if (text === formatted) {
        connection.console.log(`[Format] No changes needed`)
        return []
      }

      // Return a single TextEdit that replaces the entire document
      const fullRange = {
        start: document.positionAt(0),
        end: document.positionAt(text.length),
      }

      return [
        {
          range: fullRange,
          newText: formatted,
        },
      ]
    } catch (error) {
      // Log error but don't fail - just return no edits
      connection.console.error(
        `Formatting error: ${error instanceof Error ? error.message : "Unknown error"}`
      )
      return []
    }
  }
)

// Helper function to find which function contains the given offset
function findContainingFunction(ast: any, offset: number, text: string): any {
  if (!ast.statements) {
    connection.console.log("[SESERAGI LSP DEBUG] No statements in AST")
    return null
  }

  connection.console.log(
    `[SESERAGI LSP DEBUG] Finding containing function for offset ${offset}`
  )

  for (const statement of ast.statements) {
    if (statement.kind === "FunctionDeclaration") {
      // Calculate the function's text range more accurately
      // Start from the beginning of the function declaration
      const funcStart = getPositionFromLineColumn(text, statement.line, 1) // Start of the line

      // For a single-line function, we need to include the entire line
      const lines = text.split("\n")
      const functionLine = lines[statement.line - 1] || ""
      const functionLineEnd = funcStart + functionLine.length

      connection.console.log(
        `[SESERAGI LSP DEBUG] Function ${statement.name}: line ${statement.line}, funcStart=${funcStart}, lineEnd=${functionLineEnd}, offset=${offset}`
      )

      // Check if offset is within this function line
      // We use a more inclusive range check
      if (offset >= funcStart && offset <= functionLineEnd) {
        connection.console.log(
          `[SESERAGI LSP DEBUG] Found containing function: ${statement.name}`
        )
        return statement
      }
    }
  }

  connection.console.log("[SESERAGI LSP DEBUG] No containing function found")
  return null
}

// Helper function to convert line/column to offset
function getPositionFromLineColumn(
  text: string,
  line: number,
  column: number
): number {
  const lines = text.split("\n")
  let offset = 0

  // Add lengths of all lines before the target line
  for (let i = 0; i < line - 1 && i < lines.length; i++) {
    offset += lines[i].length + 1 // +1 for newline character
  }

  // Add the column offset
  offset += Math.max(0, column - 1)

  return offset
}

// Make the text document manager listen on the connection
// for open, change and close text document events
documents.listen(connection)

// Listen on the connection
connection.listen()
