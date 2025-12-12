/**
 * Statement generators - 文生成モジュール
 */

// Destructuring statement generators
export {
  generateRecordDestructuring,
  generateStructDestructuring,
  generateTupleDestructuring,
} from "./destructuring"
// Statement dispatcher
export { generateStatement } from "./dispatcher"
// Individual statement generators
export { generateExpressionStatement } from "./expression-statement"
export { generateFunctionDeclaration } from "./function-declaration"
export { generateImplBlock } from "./impl-block"
export { generateImportDeclaration } from "./import-declaration"
export { generateStructDeclaration } from "./struct-declaration"
export { generateTypeAliasDeclaration } from "./type-alias-declaration"
export { generateTypeDeclaration } from "./type-declaration"
export { generateVariableDeclaration } from "./variable-declaration"
