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
    id: "effect-sequence",
    label: "do: 順番に実行",
    sourcePath: "examples/spec/artifacts/schema-1/effect-do-multiple/main.ssrg",
    stdin: "",
    expectedOutput: "one\ntwo",
  },
  {
    id: "effect-pure-let",
    label: "do: pure let",
    sourcePath: "examples/spec/artifacts/schema-1/effect-do-pure-let/main.ssrg",
    stdin: "",
    expectedOutput: "hello",
  },
  {
    id: "effect-bind",
    label: "do: bind",
    sourcePath: "examples/spec/artifacts/schema-1/effect-do-bind/main.ssrg",
    stdin: "",
    expectedOutput: "loading... done",
  },
]
