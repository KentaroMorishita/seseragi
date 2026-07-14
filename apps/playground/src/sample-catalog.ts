export type PlaygroundSampleDefinition = {
  readonly id: string
  readonly label: string
  readonly sourcePath: string
  readonly stdin: string
  readonly expectedOutput: string
}

export const sampleCatalog: readonly PlaygroundSampleDefinition[] = [
  {
    id: "hello-world",
    label: "Hello world",
    sourcePath: "examples/spec/lessons/01-hello-world.ssrg",
    stdin: "",
    expectedOutput: "Hello, Seseragi!",
  },
  {
    id: "rock-paper-scissors",
    label: "型付きじゃんけん",
    sourcePath:
      "examples/spec/artifacts/schema-1/rock-paper-scissors-cli/main.ssrg",
    stdin: "rock\nscissors\n",
    expectedOutput: "Player 1 wins!",
  },
  {
    id: "mini-adventure",
    label: "ミニ冒険",
    sourcePath: "examples/spec/playground/01-mini-adventure.ssrg",
    stdin: "forest\nrope\n",
    expectedOutput: "You cross the ravine and discover a hidden lake!",
  },
  {
    id: "shipping-advisor",
    label: "送料アドバイザー",
    sourcePath: "examples/spec/playground/02-shipping-advisor.ssrg",
    stdin: "member\nexpress\n",
    expectedOutput: "Member express shipping: 500 yen",
  },
  {
    id: "seseragi-quiz",
    label: "Seseragiクイズ",
    sourcePath: "examples/spec/playground/03-seseragi-quiz.ssrg",
    stdin: "rust\n",
    expectedOutput: "Correct! The new compiler is implemented in Rust.",
  },
  {
    id: "array-scoreboard",
    label: "Arrayスコア集計",
    sourcePath: "examples/spec/playground/04-array-scoreboard.ssrg",
    stdin: "",
    expectedOutput: "Score total: 108 — perfect checksum!",
  },
]
