export function formatSeseragiCode(code: string): string {
  const lines = code.split('\n');
  const formatted: string[] = [];
  let indentLevel = 0;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const trimmedLine = line.trim();

    // 空行の処理
    if (trimmedLine === '') {
      // 連続する空行は1つまで
      if (formatted.length > 0 && formatted[formatted.length - 1] !== '') {
        formatted.push('');
      }
      continue;
    }

    // コメント行の処理
    if (trimmedLine.startsWith('//')) {
      formatted.push(trimmedLine);
      continue;
    }

    // インデントレベルの調整（処理前）
    if (trimmedLine.startsWith('}') || 
        (trimmedLine.startsWith('|') && !trimmedLine.includes('=')) ||
        trimmedLine === 'else') {
      indentLevel = Math.max(0, indentLevel - 1);
    }

    // 行をフォーマットしてインデント付きで追加
    const formattedLine = formatLine(trimmedLine, indentLevel);
    formatted.push(formattedLine);

    // インデントレベルの調整（処理後）
    if (shouldIncreaseIndent(trimmedLine)) {
      indentLevel++;
    }
  }

  return formatted.join('\n') + '\n';
}

function formatLine(line: string, indentLevel: number): string {
  const indent = '  '.repeat(indentLevel);
  
  // パイプで始まる行（match caseなど）
  if (line.startsWith('|')) {
    return indent + normalizeSpacing(line);
  }

  // その他の行
  return indent + normalizeSpacing(line);
}

function shouldIncreaseIndent(line: string): boolean {
  // 関数定義で等号で終わる場合
  if ((line.startsWith('fn ') || line.startsWith('effectful fn ')) && line.endsWith(' =')) {
    return true;
  }
  
  // type定義で等号で終わる場合
  if (line.startsWith('type ') && line.endsWith(' =')) {
    return true;
  }
  
  // match式の場合
  if (line.includes('match ') && !line.includes('|')) {
    return true;
  }
  
  // ブロック開始
  if (line.endsWith('{')) {
    return true;
  }
  
  // if-then-else のelse部分（複数行の場合）
  if (line.includes(' then') && !line.includes(' else')) {
    return true;
  }
  
  return false;
}

function normalizeSpacing(line: string): string {
  // 基本的な空白正規化のみ
  return line
    .replace(/\s+/g, ' ')
    .trim();
}

export function removeExtraWhitespace(code: string): string {
  return code
    // 複数の連続する空白を1つに
    .replace(/[ \t]+/g, ' ')
    // 行末の空白を除去
    .replace(/[ \t]+$/gm, '')
    // 複数の連続する改行を最大2つに
    .replace(/\n{3,}/g, '\n\n')
    // ファイル先頭・末尾の余分な改行を除去
    .replace(/^\n+/, '')
    .replace(/\n+$/, '\n');
}

export function normalizeOperatorSpacing(code: string): string {
  return code
    .split('\n')
    .map(line => {
      // コメント行はそのまま
      if (line.trim().startsWith('//')) {
        return line;
      }
      
      // 空行はそのまま
      if (line.trim() === '') {
        return line;
      }
      
      // インデントを保持して正規化
      const leadingSpaces = line.match(/^(\s*)/)?.[1] || '';
      const content = line.trim();
      
      return leadingSpaces + normalizeSpacing(content);
    })
    .join('\n');
}