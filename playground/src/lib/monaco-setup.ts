import type { Monaco } from "@monaco-editor/react"

export function setupSeseragiLanguage(monaco: Monaco) {
  // Seseragi言語の登録
  monaco.languages.register({ id: "seseragi" })

  // トークナイザー設定（簡易版）
  monaco.languages.setMonarchTokensProvider("seseragi", {
    keywords: [
      "fn",
      "let",
      "if",
      "then",
      "else",
      "match",
      "with",
      "end",
      "type",
      "struct",
      "impl",
      "for",
      "where",
      "do",
      "return",
      "import",
      "from",
      "as",
      "is",
      "promise",
      "resolve",
      "reject",
    ],

    builtinTypes: [
      "Int",
      "Float",
      "String",
      "Bool",
      "Char",
      "Unit",
      "List",
      "Array",
      "Maybe",
      "Either",
      "Result",
    ],

    tokenizer: {
      root: [
        // 識別子とキーワード
        [
          /[a-z_]\w*/,
          {
            cases: {
              "@keywords": "keyword",
              "@default": "identifier",
            },
          },
        ],

        // 型名（大文字始まり）
        [
          /[A-Z]\w*/,
          {
            cases: {
              "@builtinTypes": "type.identifier",
              "@default": "type",
            },
          },
        ],

        // コメント
        [/\/\/.*$/, "comment"],
        [/\/\*/, "comment", "@comment"],

        // 文字列
        [/"([^"\\]|\\.)*$/, "string.invalid"],
        [/"/, "string", "@string"],

        // 数値
        [/\d+\.\d+/, "number.float"],
        [/\d+/, "number"],

        // 演算子（具体的に定義）
        [/==|<=|>=|!=/, "operator"],
        [/&&|\|\|/, "operator"],
        [/\+\+/, "operator"],
        [/\|>|<\|/, "operator"],
        [/>>=|>>>/, "operator"],
        [/<\*>|<\$>|<@>/, "operator"],
        [/::/, "operator"],
        [/->/, "operator"],
        [/[+\-*/%=><:!~?@]/, "operator"],

        // デリミタ
        [/[{}()[\]]/, "@brackets"],
        [/[,;]/, "delimiter"],
      ],

      comment: [
        [/[^/*]+/, "comment"],
        [/\*\//, "comment", "@pop"],
        [/[/*]/, "comment"],
      ],

      string: [
        [/[^\\"]+/, "string"],
        [/\\./, "string.escape.invalid"],
        [/"/, "string", "@pop"],
      ],
    },
  })

  // 言語設定
  monaco.languages.setLanguageConfiguration("seseragi", {
    comments: {
      lineComment: "//",
      blockComment: ["/*", "*/"],
    },
    brackets: [
      ["{", "}"],
      ["[", "]"],
      ["(", ")"],
    ],
    autoClosingPairs: [
      { open: "{", close: "}" },
      { open: "[", close: "]" },
      { open: "(", close: ")" },
      { open: '"', close: '"', notIn: ["string"] },
    ],
    surroundingPairs: [
      { open: "{", close: "}" },
      { open: "[", close: "]" },
      { open: "(", close: ")" },
      { open: '"', close: '"' },
    ],
  })
}
