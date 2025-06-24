import * as fs from 'fs';
import * as path from 'path';
import * as os from 'os';
import { spawn } from 'child_process';
import { Parser } from '../parser.js';
import { generateTypeScript } from '../codegen.js';

export interface RunOptions {
  input: string;
  tempDir?: string;
  keepTemp?: boolean;
}

export async function runCommand(options: RunOptions): Promise<void> {
  const tempFile = await compileToTemp(options);
  
  try {
    await executeTypeScript(tempFile);
  } finally {
    // 一時ファイルのクリーンアップ
    if (!options.keepTemp && fs.existsSync(tempFile)) {
      fs.unlinkSync(tempFile);
    }
  }
}

async function compileToTemp(options: RunOptions): Promise<string> {
  // ファイルの存在チェック
  if (!fs.existsSync(options.input)) {
    throw new Error(`Input file not found: ${options.input}`);
  }

  // 一時ディレクトリの決定
  const tempDir = options.tempDir || os.tmpdir();
  if (!fs.existsSync(tempDir)) {
    fs.mkdirSync(tempDir, { recursive: true });
  }

  // 一時ファイル名の生成
  const inputName = path.parse(options.input).name;
  const timestamp = Date.now();
  const tempFileName = `seseragi_${inputName}_${timestamp}.ts`;
  const tempFilePath = path.join(tempDir, tempFileName);

  // ソースコードを読み込み
  const sourceCode = fs.readFileSync(options.input, 'utf-8');
  
  // パースしてASTを生成
  console.log(`Parsing ${options.input}...`);
  const parser = new Parser(sourceCode);
  const ast = parser.parse();
  
  // TypeScriptコードを生成
  console.log('Generating TypeScript code...');
  const typeScriptCode = generateTypeScript(ast.statements, {
    generateComments: false, // 実行用なのでコメントは不要
    useArrowFunctions: true,
  });
  
  // 一時ファイルに書き込み
  fs.writeFileSync(tempFilePath, typeScriptCode, 'utf-8');
  
  if (options.keepTemp) {
    console.log(`✓ Compiled to temporary file: ${tempFilePath}`);
  }
  
  return tempFilePath;
}

async function executeTypeScript(tempFile: string): Promise<void> {
  console.log('Running...');
  console.log(''); // 空行でコンパイル出力と実行結果を分離
  
  return new Promise((resolve, reject) => {
    const bunProcess = spawn('bun', [tempFile], {
      stdio: 'inherit', // 標準入出力を親プロセスと共有
    });

    bunProcess.on('close', (code) => {
      if (code === 0) {
        resolve();
      } else {
        reject(new Error(`Execution failed with exit code ${code}`));
      }
    });

    bunProcess.on('error', (error) => {
      reject(new Error(`Failed to start bun: ${error.message}`));
    });
  });
}

// プロセス終了時のクリーンアップ
const tempFiles: string[] = [];

export function addTempFile(filePath: string): void {
  tempFiles.push(filePath);
}

export function cleanupTempFiles(): void {
  tempFiles.forEach(file => {
    try {
      if (fs.existsSync(file)) {
        fs.unlinkSync(file);
      }
    } catch (error) {
      // クリーンアップエラーは無視
    }
  });
  tempFiles.length = 0;
}

// プロセス終了時のクリーンアップを登録
process.on('exit', cleanupTempFiles);
process.on('SIGINT', () => {
  cleanupTempFiles();
  process.exit(0);
});
process.on('SIGTERM', () => {
  cleanupTempFiles();
  process.exit(0);
});