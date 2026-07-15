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
  {
    id: "trait-badges",
    label: "Traitバッジ",
    sourcePath: "examples/spec/playground/05-trait-badges.ssrg",
    stdin: "",
    expectedOutput: "Status badge: active\nMode badge: automatic",
  },
  {
    id: "generic-instance",
    label: "Generic instance",
    sourcePath: "examples/spec/playground/06-generic-instance.ssrg",
    stdin: "",
    expectedOutput: "Generic Maybe: empty\nGeneric Maybe: present",
  },
  {
    id: "constrained-instance",
    label: "Constraint付きinstance",
    sourcePath: "examples/spec/playground/07-constrained-instance.ssrg",
    stdin: "",
    expectedOutput: "Constrained Maybe: empty\nBadge is ready",
  },
  {
    id: "constrained-function",
    label: "Constraint付きgeneric関数",
    sourcePath: "examples/spec/playground/08-constrained-function.ssrg",
    stdin: "",
    expectedOutput: "Badge is ready\nDevice is online",
  },
  {
    id: "method-constraint",
    label: "Method固有constraint",
    sourcePath: "examples/spec/playground/09-method-constraint.ssrg",
    stdin: "",
    expectedOutput: "Badge label: active\nDevice label: online",
  },
]
