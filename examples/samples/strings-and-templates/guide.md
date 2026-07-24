Double-quoted Stringは、source上のescapeを実際の文字へ復号します。

- `\n`、`\r`、`\t`は改行・復帰・tabです。
- `\\`と`\"`はbackslashとdouble quoteです。
- `\u{03BB}`のようなUnicode scalar escapeも使えます。
- 未定義escapeや不正なUnicode値は、実行前に`SES-P0201`で報告されます。
- 値を埋め込むときはtemplateの`${...}`を使います。
