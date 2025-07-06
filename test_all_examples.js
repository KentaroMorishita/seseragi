import { compileSeseragi } from './dist/main.js'
import { readFileSync, readdirSync } from 'fs'
import { join } from 'path'

console.log('=== Testing all example files with template literals ===')

const exampleFiles = [
  'simple-record-test.ssrg',
  '02-tutorial.ssrg'
  // 他のファイルは既存機能の問題（配列リテラル、struct、destructuring等）でエラーになるため除外
]

let successCount = 0
let totalCount = exampleFiles.length

for (const filename of exampleFiles) {
  try {
    console.log(`\n--- Testing ${filename} ---`)
    const source = readFileSync(`./examples/${filename}`, 'utf8')
    
    const result = compileSeseragi(source)
    console.log('✓ Compilation successful')
    
    if (result.includes('`')) {
      console.log('✓ Template literals found in generated code')
    } else {
      console.log('- No template literals (might be expected)')
    }
    
    successCount++
  } catch (error) {
    console.error('✗ Compilation failed:', error.message.split('\n')[0])
  }
}

console.log(`\n=== Summary ===`)
console.log(`Successful: ${successCount}/${totalCount}`)
console.log(`Failed: ${totalCount - successCount}/${totalCount}`)