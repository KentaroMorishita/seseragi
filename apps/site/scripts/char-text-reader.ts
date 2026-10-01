import { canonicalExample } from "./canonical-example"

// Exact identities and independently executed results; prose is in typed locale modules.
export const charTextCases = [
  {
    slug: "codepoint",
    name: "codePoint",
    module: "std/char",
    identity: "std/char::codePoint",
    route: "/docs/library/char/function/codepoint/",
    output: "symbol: 128077\nletter: 65\nnull: 0\n",
    typescriptOutput:
      "symbol: 128077\nletter: 65\nnull: 0\nfirst UTF-16 unit: 55357\nUTF-16 units: 2\n",
  },
  {
    slug: "fromcodepoint",
    name: "fromCodePoint",
    module: "std/char",
    identity: "std/char::fromCodePoint",
    route: "/docs/library/char/function/fromcodepoint/",
    output:
      "found: 👍\nnot a scalar\nnot a scalar\nnot a scalar\nnot a scalar\nzero accepted: True\nlast accepted: True\n",
    typescriptOutput:
      "found: 👍\nnot a scalar\nnot a scalar\nnot a scalar\nnot a scalar\nzero accepted: true\nlast accepted: true\nJS accepts surrogate: 55296\n",
  },
  {
    slug: "tostring",
    name: "toString",
    module: "std/char",
    identity: "std/char::toString",
    route: "/docs/library/char/function/tostring/",
    output: "Status: 👍\nscalars: 1\nUTF-8 bytes: 4\n",
    typescriptOutput: "Status: 👍\nscalars: 1\nUTF-8 bytes: 4\n",
  },
  {
    slug: "trimstart",
    name: "trimStart",
    module: "std/text",
    identity: "std/text::trimStart",
    route: "/docs/library/text/function/trimstart/",
    output:
      "result: [Aki  ]\noriginal: [  Aki  ]\nNEL removed: True\nFEFF preserved: True\nempty: []\nonly whitespace: []\n",
    typescriptOutput:
      "result: [Aki  ]\noriginal: [  Aki  ]\nJS removes NEL: false\nUnicode removes NEL: true\nJS preserves FEFF: false\nUnicode preserves FEFF: true\nempty: []\nonly whitespace: []\n",
  },
  {
    slug: "trimend",
    name: "trimEnd",
    module: "std/text",
    identity: "std/text::trimEnd",
    route: "/docs/library/text/function/trimend/",
    output:
      "result: [  Aki]\noriginal: [  Aki  ]\nNEL removed: True\nFEFF preserved: True\nempty: []\nonly whitespace: []\n",
    typescriptOutput:
      "result: [  Aki]\noriginal: [  Aki  ]\nJS removes NEL: false\nUnicode removes NEL: true\nJS preserves FEFF: false\nUnicode preserves FEFF: true\nempty: []\nonly whitespace: []\n",
  },
  {
    slug: "words",
    name: "words",
    module: "std/text",
    identity: "std/text::words",
    route: "/docs/library/text/function/words/",
    output:
      'pieces: ["hello,","世界!"]\njoined: hello, | 世界!\nJapanese: ["東京都渋谷区"]\nempty: []\nonly whitespace: []\nFEFF preserved: True\n',
    typescriptOutput:
      'pieces: ["hello,","世界!"]\njoined: hello, | 世界!\nJapanese: ["東京都渋谷区"]\nempty: []\nonly whitespace: []\nFEFF preserved: true\nJS \\s treats NEL as whitespace: false\nJS \\s treats FEFF as whitespace: true\n',
  },
  {
    slug: "generalcategory",
    name: "generalCategory",
    module: "std/text/unicode",
    identity: "std/text/unicode::generalCategory",
    route: "/docs/library/text/unicode/function/generalcategory/",
    output:
      "A category: UppercaseLetter\nA uppercase: True\n٣ category: DecimalNumber\nU+0301 category: NonspacingMark\nU+FEFF category: Format\nU+10FFFF category: Unassigned\n",
    typescriptOutput:
      "A uppercase: true\n٣ uppercase: false\nU+0301 uppercase: false\nU+FEFF uppercase: false\nU+10FFFF uppercase: false\n",
  },
  {
    slug: "isalphabetic",
    name: "isAlphabetic",
    module: "std/text/unicode",
    identity: "std/text/unicode::isAlphabetic",
    route: "/docs/library/text/unicode/function/isalphabetic/",
    output:
      "A alphabetic: True\n漢 alphabetic: True\n3 alphabetic: False\nU+0345 alphabetic: True\nU+0301 alphabetic: False\n",
    typescriptOutput:
      "A alphabetic: true\n漢 alphabetic: true\n3 alphabetic: false\nU+0345 alphabetic: true\nU+0301 alphabetic: false\n",
  },
  {
    slug: "isdecimaldigit",
    name: "isDecimalDigit",
    module: "std/text/unicode",
    identity: "std/text/unicode::isDecimalDigit",
    route: "/docs/library/text/unicode/function/isdecimaldigit/",
    output:
      "3 decimal: True\n٣ decimal: True\n３ decimal: True\n² decimal: False\nⅣ decimal: False\n",
    typescriptOutput:
      "3 decimal: true\n٣ decimal: true\n３ decimal: true\n² decimal: false\nⅣ decimal: false\n",
  },
  {
    slug: "ismark",
    name: "isMark",
    module: "std/text/unicode",
    identity: "std/text/unicode::isMark",
    route: "/docs/library/text/unicode/function/ismark/",
    output:
      "acute accent: True\nU+093E mark: True\nU+20DD mark: True\nA mark: False\né mark: False\n🏽 mark: False\nU+200D mark: False\n",
    typescriptOutput:
      "acute accent: true\nU+093E mark: true\nU+20DD mark: true\nA mark: false\né mark: false\n🏽 mark: false\nU+200D mark: false\n",
  },
  {
    slug: "iswhitespace",
    name: "isWhitespace",
    module: "std/text/unicode",
    identity: "std/text/unicode::isWhitespace",
    route: "/docs/library/text/unicode/function/iswhitespace/",
    output:
      "space whitespace: True\ntab whitespace: True\nNEL whitespace: True\nEM SPACE whitespace: True\nU+FEFF whitespace: False\nU+200B whitespace: False\n",
    typescriptOutput:
      "space whitespace: true\ntab whitespace: true\nNEL whitespace: true\nEM SPACE whitespace: true\nU+FEFF whitespace: false\nU+200B whitespace: false\n",
  },
  {
    slug: "simplecasefold",
    name: "simpleCaseFold",
    module: "std/text/unicode",
    identity: "std/text/unicode::simpleCaseFold",
    route: "/docs/library/text/unicode/function/simplecasefold/",
    output:
      "same scalar: False\nsame simple fold: True\ncapital fold: σ\nfinal fold: σ\nsharp s: ß\ncapital sharp s: ß\ndotted I: İ\ndotted I equals i: False\n",
    typescriptOutput:
      "same scalar: false\ncase-insensitive sigma: true\ncase-insensitive sharp s: true\ndotted I equals i: false\n",
  },
] as const

export function charTextExamples(playgroundUrl: string) {
  return charTextCases.flatMap(({ slug }) => [
    canonicalExample(
      `char-text-${slug}`,
      `apps/site/examples/src/api-char-text/${slug}.ssrg`,
      playgroundUrl
    ),
    canonicalExample(
      `char-text-${slug}-typescript`,
      `apps/site/examples/src/api-char-text/${slug}.ts`,
      playgroundUrl,
      false,
      "typescript"
    ),
  ])
}
