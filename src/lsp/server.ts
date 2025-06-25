import {
  createConnection,
  TextDocuments,
  Diagnostic,
  DiagnosticSeverity,
  ProposedFeatures,
  InitializeParams,
  DidChangeConfigurationNotification,
  CompletionItem,
  CompletionItemKind,
  TextDocumentPositionParams,
  TextDocumentSyncKind,
  InitializeResult,
  Hover,
  MarkupKind,
  DocumentFormattingParams,
  TextEdit,
  DefinitionParams,
  Definition,
  Location,
} from "vscode-languageserver/node";

import { TextDocument } from "vscode-languageserver-textdocument";
import { URI } from "vscode-uri";
import { Lexer } from "../formatter/lexer";
import { Parser } from "../parser";
import { TypeChecker } from "../typechecker";
import { SeseragiFormatter, defaultFormatterOptions } from "../formatter/formatter";

// Create a connection for the server, using Node's IPC as a transport
const connection = createConnection(ProposedFeatures.all);

// Create a text document manager
const documents: TextDocuments<TextDocument> = new TextDocuments(TextDocument);

let hasConfigurationCapability = false;
let hasWorkspaceFolderCapability = false;
let hasDiagnosticRelatedInformationCapability = false;

connection.onInitialize((params: InitializeParams) => {
  connection.console.log('Seseragi Language Server starting...');
  const capabilities = params.capabilities;

  // Does the client support the `workspace/configuration` request?
  hasConfigurationCapability = !!(
    capabilities.workspace && !!capabilities.workspace.configuration
  );
  hasWorkspaceFolderCapability = !!(
    capabilities.workspace && !!capabilities.workspace.workspaceFolders
  );
  hasDiagnosticRelatedInformationCapability = !!(
    capabilities.textDocument &&
    capabilities.textDocument.publishDiagnostics &&
    capabilities.textDocument.publishDiagnostics.relatedInformation
  );

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
  };
  if (hasWorkspaceFolderCapability) {
    result.capabilities.workspace = {
      workspaceFolders: {
        supported: true,
      },
    };
  }
  return result;
});

connection.onInitialized(() => {
  if (hasConfigurationCapability) {
    // Register for all configuration changes.
    connection.client.register(
      DidChangeConfigurationNotification.type,
      undefined,
    );
  }
  if (hasWorkspaceFolderCapability) {
    connection.workspace.onDidChangeWorkspaceFolders((_event) => {
      connection.console.log("Workspace folder change event received.");
    });
  }
});

// The example settings
interface SeseragiSettings {
  maxNumberOfProblems: number;
}

// The global settings, used when the `workspace/configuration` request is not supported by the client.
// Please note that this is not the case when using this server with the client provided in this example
// but could happen with other clients.
const defaultSettings: SeseragiSettings = { maxNumberOfProblems: 1000 };
let globalSettings: SeseragiSettings = defaultSettings;

// Cache the settings of all open documents
const documentSettings: Map<string, Thenable<SeseragiSettings>> = new Map();

connection.onDidChangeConfiguration((change) => {
  if (hasConfigurationCapability) {
    // Reset all cached document settings
    documentSettings.clear();
  } else {
    globalSettings = <SeseragiSettings>(
      (change.settings.seseragi || defaultSettings)
    );
  }

  // Revalidate all open text documents
  documents.all().forEach(validateTextDocument);
});

function getDocumentSettings(resource: string): Thenable<SeseragiSettings> {
  if (!hasConfigurationCapability) {
    return Promise.resolve(globalSettings);
  }
  let result = documentSettings.get(resource);
  if (!result) {
    result = connection.workspace.getConfiguration({
      scopeUri: resource,
      section: "seseragi",
    });
    documentSettings.set(resource, result);
  }
  return result;
}

// Only keep settings for open documents
documents.onDidClose((e) => {
  documentSettings.delete(e.document.uri);
});

// The content of a text document has changed. This event is emitted
// when the text document first opened or when its content has changed.
documents.onDidChangeContent((change) => {
  validateTextDocument(change.document);
});

async function validateTextDocument(textDocument: TextDocument): Promise<void> {
  // In this simple example we get the settings for every validate run.
  const settings = await getDocumentSettings(textDocument.uri);

  // The validator creates diagnostics for all uppercase words longer than 2 characters
  const text = textDocument.getText();
  const diagnostics: Diagnostic[] = [];

  try {
    // Parse the document
    const parser = new Parser(text);
    const ast = parser.parse();

    // Type check the document
    const typeChecker = new TypeChecker();
    const typeErrors = typeChecker.check(ast);

    // Convert type errors to diagnostics
    for (const error of typeErrors) {
      const startPos = error.line !== undefined && error.column !== undefined
        ? { line: error.line, character: error.column }
        : textDocument.positionAt(0);
      const endPos = error.line !== undefined && error.column !== undefined
        ? { line: error.line, character: error.column + ((error as any).length || 1) }
        : textDocument.positionAt(Math.min(text.length, 100));
        
      const diagnostic: Diagnostic = {
        severity: DiagnosticSeverity.Error,
        range: {
          start: startPos,
          end: endPos,
        },
        message: error.message,
        source: "seseragi",
      };
      
      if (hasDiagnosticRelatedInformationCapability) {
        diagnostic.relatedInformation = [
          {
            location: {
              uri: textDocument.uri,
              range: Object.assign({}, diagnostic.range),
            },
            message: "Type error occurred here",
          },
        ];
      }
      diagnostics.push(diagnostic);
    }
  } catch (error) {
    // Handle lexer/parser errors
    let errorMessage = "Unknown parsing error";
    let startPos = 0;
    let endPos = Math.min(text.length, 100);
    
    if (error instanceof Error) {
      errorMessage = error.message;
      // Try to extract position information from error message if available
      const posMatch = error.message.match(/line (\d+), column (\d+)/);
      if (posMatch) {
        const line = parseInt(posMatch[1], 10) - 1; // Convert to 0-based
        const col = parseInt(posMatch[2], 10) - 1;
        startPos = textDocument.offsetAt({ line, character: col });
        endPos = Math.min(startPos + 10, text.length);
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
    };
    diagnostics.push(diagnostic);
  }

  // Send the computed diagnostics to VSCode.
  connection.sendDiagnostics({ uri: textDocument.uri, diagnostics });
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
    const document = documents.get(textDocumentPosition.textDocument.uri);
    if (!document) {
      return [];
    }

    const text = document.getText();
    const position = textDocumentPosition.position;
    const offset = document.offsetAt(position);
    
    // Get current line text for context
    const lineText = text.split('\n')[position.line] || '';
    const beforeCursor = lineText.substring(0, position.character);
    
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
    ];

    // Add contextual completions based on current line
    if (beforeCursor.trim().startsWith('type ')) {
      completionItems.push(
        {
          label: 'Maybe<T>',
          kind: CompletionItemKind.TypeParameter,
          data: 100,
          insertText: 'Maybe<$1>',
          insertTextFormat: 2, // Snippet format
        },
        {
          label: 'Either<L,R>',
          kind: CompletionItemKind.TypeParameter,
          data: 101,
          insertText: 'Either<$1, $2>',
          insertTextFormat: 2,
        }
      );
    }

    return completionItems;
  },
);

// This handler resolves additional information for the item selected in
// the completion list.
connection.onCompletionResolve((item: CompletionItem): CompletionItem => {
  if (item.data === 1) {
    item.detail = "Function definition";
    item.documentation = "Define a new function";
  } else if (item.data === 2) {
    item.detail = "Variable binding";
    item.documentation = "Bind a value to an immutable variable";
  } else if (item.data === 3) {
    item.detail = "Type definition";
    item.documentation = "Define a new type";
  } else if (item.data === 4) {
    item.detail = "Pattern matching";
    item.documentation = "Pattern match on algebraic data types";
  } else if (item.data === 5) {
    item.detail = "Effectful function";
    item.documentation = "Mark a function as having side effects";
  } else if (item.data === 6) {
    item.detail = "Operator definition";
    item.documentation = "Define a custom operator";
  } else if (item.data === 7) {
    item.detail = "Implementation block";
    item.documentation = "Implement methods for a type";
  } else if (item.data === 8) {
    item.detail = "Maybe<T>";
    item.documentation = "Optional value type";
  } else if (item.data === 9) {
    item.detail = "Either<L,R>";
    item.documentation = "Error handling type";
  } else if (item.data === 10) {
    item.detail = "IO<T>";
    item.documentation = "IO monad for side effects";
  }
  return item;
});

// This handler provides hover information.
connection.onHover((params: TextDocumentPositionParams): Hover | null => {
  const document = documents.get(params.textDocument.uri);
  if (!document) {
    return null;
  }

  const text = document.getText();
  const position = params.position;
  const offset = document.offsetAt(position);

  try {
    // Parse the document
    const parser = new Parser(text);
    const ast = parser.parse();

    // Get hover information from the position
    const hoverInfo = getHoverInfo(ast, offset, text);
    
    if (hoverInfo) {
      return {
        contents: {
          kind: MarkupKind.Markdown,
          value: hoverInfo
        }
      };
    }
  } catch (error) {
    // Silently ignore errors for hover
    connection.console.log(`Hover error: ${error instanceof Error ? error.message : 'Unknown error'}`);
  }

  return null;
});

// Helper function to get hover information at a specific offset
function getHoverInfo(ast: any, offset: number, text: string): string | null {
  // Find the token/node at the given offset
  const wordAtPosition = getWordAtPosition(text, offset);
  if (!wordAtPosition) {
    return null;
  }

  // Try to get type information using the type checker
  try {
    const typeChecker = new TypeChecker();
    const typeInfo = getTypeInfoWithTypeChecker(ast, wordAtPosition, typeChecker);
    if (typeInfo) {
      return typeInfo;
    }
  } catch (error) {
    // Fall back to basic type info if type checker fails
  }

  // Try to find type information for the word
  const typeInfo = getTypeInfoForSymbol(ast, wordAtPosition);
  if (typeInfo) {
    return `**${wordAtPosition}**: ${typeInfo}`;
  }

  // Return basic information for known keywords
  const keywordInfo = getKeywordInfo(wordAtPosition);
  if (keywordInfo) {
    return keywordInfo;
  }

  return null;
}

// Get type information using the type checker
function getTypeInfoWithTypeChecker(ast: any, symbol: string, typeChecker: TypeChecker): string | null {
  // Run type checking to populate type environment
  typeChecker.check(ast);
  
  // Look for the symbol in the AST with type information
  const symbolInfo = findSymbolWithType(ast, symbol, typeChecker);
  if (symbolInfo) {
    return formatTypeInfo(symbol, symbolInfo);
  }
  
  return null;
}

// Find symbol with type information from the type checker
function findSymbolWithType(ast: any, symbol: string, typeChecker: TypeChecker): any {
  if (!ast.statements) {
    return null;
  }

  for (const statement of ast.statements) {
    if (statement.kind === 'FunctionDeclaration' && statement.name === symbol) {
      return {
        type: 'function',
        name: symbol,
        parameters: statement.parameters,
        returnType: statement.returnType,
        isEffectful: statement.isEffectful
      };
    }
    
    if (statement.kind === 'VariableDeclaration' && statement.name === symbol) {
      return {
        type: 'variable',
        name: symbol,
        varType: statement.type,
        value: statement.initializer
      };
    }
    
    if (statement.kind === 'TypeDeclaration' && statement.name === symbol) {
      return {
        type: 'type',
        name: symbol,
        definition: statement.definition
      };
    }
  }
  
  return null;
}

// Format type information for hover display
function formatTypeInfo(symbol: string, info: any): string {
  switch (info.type) {
    case 'function':
      const params = info.parameters?.map((p: any) => {
        const paramType = formatTypeForDisplay(p.type);
        return `${p.name}: ${paramType}`;
      }).join(', ') || '';
      
      const returnType = formatTypeForDisplay(info.returnType);
      const effectful = info.isEffectful ? 'effectful ' : '';
      
      return `\`\`\`seseragi\n${effectful}fn ${symbol}(${params}) -> ${returnType}\n\`\`\``;
      
    case 'variable':
      const varType = formatTypeForDisplay(info.varType);
      return `\`\`\`seseragi\nlet ${symbol}: ${varType}\n\`\`\``;
      
    case 'type':
      return `\`\`\`seseragi\ntype ${symbol}\n\`\`\`\n\nUser-defined type`;
      
    default:
      return null;
  }
}

// Format type for display in hover
function formatTypeForDisplay(type: any): string {
  if (!type) return 'unknown';
  
  if (typeof type === 'string') {
    return type;
  }
  
  if (type.kind === 'PrimitiveType') {
    return type.name;
  }
  
  if (type.kind === 'FunctionType') {
    const paramType = formatTypeForDisplay(type.parameterType);
    const returnType = formatTypeForDisplay(type.returnType);
    return `${paramType} -> ${returnType}`;
  }
  
  if (type.kind === 'GenericType') {
    const baseType = formatTypeForDisplay(type.baseType);
    const typeArgs = type.typeArguments?.map(formatTypeForDisplay).join(', ') || '';
    return typeArgs ? `${baseType}<${typeArgs}>` : baseType;
  }
  
  return type.name || 'unknown';
}

// Get the word at a specific position in the text
function getWordAtPosition(text: string, offset: number): string | null {
  if (offset < 0 || offset >= text.length) {
    return null;
  }

  const before = text.substring(0, offset);
  const after = text.substring(offset);

  const beforeMatch = before.match(/[a-zA-Z_][a-zA-Z0-9_]*$/);
  const afterMatch = after.match(/^[a-zA-Z0-9_]*/);

  if (beforeMatch) {
    const start = beforeMatch[0];
    const end = afterMatch ? afterMatch[0] : "";
    return start + end;
  }

  return null;
}

// Get type information for a symbol (basic implementation)
function getTypeInfoForSymbol(ast: any, symbol: string): string | null {
  // This is a simplified implementation
  // In a full implementation, you would traverse the AST to find the symbol's type
  
  // Try to find function definitions
  if (ast.items) {
    for (const item of ast.items) {
      if (item.type === 'FunctionDefinition' && item.name === symbol) {
        const paramTypes = item.parameters?.map((p: any) => 
          p.type ? `${p.name}: ${formatType(p.type)}` : p.name
        ).join(', ') || '';
        const returnType = item.returnType ? formatType(item.returnType) : 'unknown';
        return `\`\`\`seseragi\nfn ${symbol}(${paramTypes}) -> ${returnType}\n\`\`\``;
      }
      
      if (item.type === 'VariableDefinition' && item.name === symbol) {
        const varType = item.valueType ? formatType(item.valueType) : 'inferred';
        return `\`\`\`seseragi\nlet ${symbol}: ${varType}\n\`\`\``;
      }
    }
  }

  return null;
}

// Format type information for display
function formatType(type: any): string {
  if (typeof type === 'string') {
    return type;
  }
  if (type.name) {
    return type.name;
  }
  if (type.type === 'FunctionType') {
    const params = type.parameters?.map(formatType).join(' -> ') || '';
    const returnType = type.returnType ? formatType(type.returnType) : 'unknown';
    return params ? `${params} -> ${returnType}` : returnType;
  }
  return 'unknown';
}

// Get information for known keywords
function getKeywordInfo(keyword: string): string | null {
  const keywordDocs: Record<string, string> = {
    'fn': '**Function Definition**\n\nDefines a new function.\n\n```seseragi\nfn name(param: Type) -> ReturnType = expression\n```',
    'let': '**Variable Binding**\n\nBinds a value to an immutable variable.\n\n```seseragi\nlet name: Type = expression\n```',
    'type': '**Type Definition**\n\nDefines a new algebraic data type.\n\n```seseragi\ntype Maybe<T> = Some(T) | None\n```',
    'match': '**Pattern Matching**\n\nPattern matches on algebraic data types.\n\n```seseragi\nmatch value {\n  Some(x) -> x,\n  None -> defaultValue\n}\n```',
    'effectful': '**Effectful Function**\n\nMarks a function as having side effects.\n\n```seseragi\neffectful fn print(msg: String) -> IO<Unit>\n```',
    'operator': '**Operator Definition**\n\nDefines a custom operator.\n\n```seseragi\noperator +++ (a: String, b: String) -> String = concat(a, b)\n```',
    'impl': '**Implementation Block**\n\nImplements methods for a type.\n\n```seseragi\nimpl MyType {\n  fn method(self) -> ReturnType = ...\n}\n```',
    'Maybe': '**Maybe Type**\n\nOptional value type for handling null values safely.\n\n```seseragi\ntype Maybe<T> = Some(T) | None\n```',
    'Either': '**Either Type**\n\nError handling type for computations that may fail.\n\n```seseragi\ntype Either<L, R> = Left(L) | Right(R)\n```',
    'IO': '**IO Monad**\n\nMonad for handling side effects in a pure functional way.\n\n```seseragi\ntype IO<T> = IO(T)\n```',
  };

  return keywordDocs[keyword] || null;
}

// This handler provides go to definition functionality.
connection.onDefinition((params: DefinitionParams): Definition | null => {
  const document = documents.get(params.textDocument.uri);
  if (!document) {
    return null;
  }

  const text = document.getText();
  const position = params.position;
  const offset = document.offsetAt(position);

  try {
    // Parse the document
    const parser = new Parser(text);
    const ast = parser.parse();

    // Get the word at the current position
    const wordAtPosition = getWordAtPosition(text, offset);
    if (!wordAtPosition) {
      return null;
    }

    // Find the definition of the symbol in the AST
    const definition = findDefinition(ast, wordAtPosition, document);
    if (definition) {
      return definition;
    }

  } catch (error) {
    connection.console.log(`Definition error: ${error instanceof Error ? error.message : 'Unknown error'}`);
  }

  return null;
});

// Helper function to find symbol definitions
function findDefinition(ast: any, symbol: string, document: TextDocument): Location | null {
  if (!ast.items) {
    return null;
  }

  for (const item of ast.items) {
    if (item.type === 'FunctionDefinition' && item.name === symbol) {
      // Calculate position from AST node if available
      const line = item.line || 0;
      const character = item.column || 0;
      return {
        uri: document.uri,
        range: {
          start: { line, character },
          end: { line, character: character + symbol.length },
        },
      };
    }
    
    if (item.type === 'VariableDefinition' && item.name === symbol) {
      const line = item.line || 0;
      const character = item.column || 0;
      return {
        uri: document.uri,
        range: {
          start: { line, character },
          end: { line, character: character + symbol.length },
        },
      };
    }
    
    if (item.type === 'TypeDefinition' && item.name === symbol) {
      const line = item.line || 0;
      const character = item.column || 0;
      return {
        uri: document.uri,
        range: {
          start: { line, character },
          end: { line, character: character + symbol.length },
        },
      };
    }
  }

  return null;
}

// This handler provides document formatting.
connection.onDocumentFormatting((params: DocumentFormattingParams): TextEdit[] => {
  const document = documents.get(params.textDocument.uri);
  if (!document) {
    return [];
  }

  try {
    const text = document.getText();
    const formatter = new SeseragiFormatter({
      ...defaultFormatterOptions,
      indentSize: params.options.tabSize,
      // Convert insertSpaces to indentSize setting if needed
    });
    
    const formattedText = formatter.format(text);
    
    // Return a single TextEdit that replaces the entire document
    const fullRange = {
      start: document.positionAt(0),
      end: document.positionAt(text.length)
    };
    
    return [{
      range: fullRange,
      newText: formattedText
    }];
  } catch (error) {
    // Log error but don't fail - just return no edits
    connection.console.error(`Formatting error: ${error instanceof Error ? error.message : 'Unknown error'}`);
    return [];
  }
});

// Make the text document manager listen on the connection
// for open, change and close text document events
documents.listen(connection);

// Listen on the connection
connection.listen();