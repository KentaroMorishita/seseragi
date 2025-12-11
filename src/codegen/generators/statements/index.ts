/**
 * Statement generators - 文生成モジュール
 */

// Statement dispatcher
export { generateStatement } from "./dispatcher"

// Individual statement generators
export { generateExpressionStatement } from "./expression-statement"
export { generateVariableDeclaration } from "./variable-declaration"

// TODO: 以下のジェネレーターを追加予定
// export { generateFunctionDeclaration } from "./function-declaration"
// export { generateTypeDeclaration } from "./type-declaration"
// export { generateTypeAliasDeclaration } from "./type-alias-declaration"
// export { generateTupleDestructuring } from "./tuple-destructuring"
// export { generateRecordDestructuring } from "./record-destructuring"
// export { generateStructDestructuring } from "./struct-destructuring"
// export { generateStructDeclaration } from "./struct-declaration"
// export { generateImplBlock } from "./impl-block"
// export { generateImportDeclaration } from "./import-declaration"
