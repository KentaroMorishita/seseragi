import { readFile, writeFile } from 'node:fs/promises';
import { formatSeseragiCode, removeExtraWhitespace, normalizeOperatorSpacing } from '../formatter/index.js';

export interface FormatCommandOptions {
  input: string;
  output?: string;
  inPlace?: boolean;
  check?: boolean;
  removeWhitespace?: boolean;
  normalizeSpacing?: boolean;
}

export async function formatCommand(options: FormatCommandOptions): Promise<void> {
  try {
    const input = await readFile(options.input, 'utf8');
    
    let formatted = input;
    
    // 不要な空白除去
    if (options.removeWhitespace) {
      formatted = removeExtraWhitespace(formatted);
    }
    
    // 演算子のスペーシング正規化
    if (options.normalizeSpacing) {
      formatted = normalizeOperatorSpacing(formatted);
    }
    
    // フル フォーマット
    formatted = formatSeseragiCode(formatted);
    
    // チェックモード：フォーマットが必要かどうかのみ確認
    if (options.check) {
      if (input !== formatted) {
        console.error(`File ${options.input} is not formatted correctly`);
        process.exit(1);
      } else {
        console.log(`File ${options.input} is correctly formatted`);
        return;
      }
    }
    
    // 出力先決定
    const outputPath = options.inPlace ? options.input : options.output;
    
    if (outputPath) {
      await writeFile(outputPath, formatted, 'utf8');
      console.log(`Formatted ${options.input} -> ${outputPath}`);
    } else {
      console.log(formatted);
    }
    
  } catch (error) {
    console.error(`Error formatting ${options.input}:`, error);
    process.exit(1);
  }
}

export async function formatMultipleFiles(
  files: string[], 
  options: Omit<FormatCommandOptions, 'input'>
): Promise<void> {
  for (const file of files) {
    await formatCommand({ ...options, input: file });
  }
}