export function formatSeseragiCode(code: string): string {
  // まずは基本的なクリーンアップのみ
  return removeExtraWhitespace(normalizeOperatorSpacing(code));
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
        return line.trim();
      }
      
      // 空行はそのまま
      if (line.trim() === '') {
        return '';
      }
      
      return line
        // 基本的な空白正規化
        .replace(/\s+/g, ' ')
        .trim();
    })
    .join('\n');
}