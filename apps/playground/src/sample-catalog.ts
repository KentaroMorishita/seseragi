export type PlaygroundSampleDefinition = {
  readonly id: string
  readonly label: string
  readonly category: "基本" | "アプリ" | "型と抽象化"
  readonly sourcePath: string
  readonly stdin: string
  readonly expectedOutput: string
}

export const sampleCatalog: readonly PlaygroundSampleDefinition[] = [
  {
    id: "hello-world",
    label: "Hello world",
    category: "基本",
    sourcePath: "examples/spec/lessons/01-hello-world.ssrg",
    stdin: "",
    expectedOutput: "Hello, Seseragi!",
  },
  {
    id: "pipeline-application",
    label: "関数適用とパイプライン",
    category: "基本",
    sourcePath:
      "examples/spec/artifacts/schema-1/pipeline-application/main.ssrg",
    stdin: "",
    expectedOutput: "Pipeline answer: 42",
  },
  {
    id: "string-add",
    label: "Stringで招待状",
    category: "基本",
    sourcePath: "examples/spec/artifacts/schema-1/string-add/main.ssrg",
    stdin: "",
    expectedOutput: "Hello, Mio! Welcome to Seseragi Night.",
  },
  {
    id: "array-scoreboard",
    label: "Arrayスコア集計",
    category: "基本",
    sourcePath: "examples/spec/playground/04-array-scoreboard.ssrg",
    stdin: "",
    expectedOutput: "Score total: 108 — perfect checksum!",
  },
  {
    id: "range-comprehension",
    label: "Rangeと内包表記",
    category: "基本",
    sourcePath:
      "examples/spec/artifacts/schema-1/range-comprehension/main.ssrg",
    stdin: "",
    expectedOutput: "even square total: 220",
  },
  {
    id: "rock-paper-scissors",
    label: "型付きじゃんけん",
    category: "アプリ",
    sourcePath:
      "examples/spec/artifacts/schema-1/rock-paper-scissors-cli/main.ssrg",
    stdin: "rock\nscissors\n",
    expectedOutput: "Player 1 wins!",
  },
  {
    id: "mini-adventure",
    label: "ミニ冒険",
    category: "アプリ",
    sourcePath: "examples/spec/playground/01-mini-adventure.ssrg",
    stdin: "forest\nrope\n",
    expectedOutput: "You cross the ravine and discover a hidden lake!",
  },
  {
    id: "shipping-advisor",
    label: "送料アドバイザー",
    category: "アプリ",
    sourcePath: "examples/spec/playground/02-shipping-advisor.ssrg",
    stdin: "member\nexpress\n",
    expectedOutput: "Member express shipping: 500 yen",
  },
  {
    id: "seseragi-quiz",
    label: "Seseragiクイズ",
    category: "アプリ",
    sourcePath: "examples/spec/playground/03-seseragi-quiz.ssrg",
    stdin: "rust\n",
    expectedOutput: "Correct! The new compiler is implemented in Rust.",
  },
  {
    id: "trait-badges",
    label: "Traitバッジ",
    category: "型と抽象化",
    sourcePath: "examples/spec/playground/05-trait-badges.ssrg",
    stdin: "",
    expectedOutput: "Status badge: active\nMode badge: automatic",
  },
  {
    id: "generic-instance",
    label: "Generic instance",
    category: "型と抽象化",
    sourcePath: "examples/spec/playground/06-generic-instance.ssrg",
    stdin: "",
    expectedOutput: "Generic Maybe: empty\nGeneric Maybe: present",
  },
  {
    id: "applicative-maybe",
    label: "Applicativeの流れ",
    category: "型と抽象化",
    sourcePath: "examples/spec/artifacts/schema-1/applicative-maybe/main.ssrg",
    stdin: "",
    expectedOutput: "Just 42",
  },
  {
    id: "monad-maybe",
    label: "Monadの流れ",
    category: "型と抽象化",
    sourcePath: "examples/spec/artifacts/schema-1/monad-maybe/main.ssrg",
    stdin: "",
    expectedOutput: "Just 42\nJust 42\nJust 44\nNothing\nJust 42\nNothing",
  },
  {
    id: "monad-either",
    label: "MonadとEither",
    category: "型と抽象化",
    sourcePath: "examples/spec/artifacts/schema-1/monad-either/main.ssrg",
    stdin: "",
    expectedOutput: "Right 42\nLeft stopped",
  },
]
