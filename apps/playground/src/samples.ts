import rockPaperScissors from "../../../examples/spec/artifacts/schema-1/rock-paper-scissors-cli/main.ssrg?raw"
import helloWorld from "../../../examples/spec/lessons/01-hello-world.ssrg?raw"

export type PlaygroundSample = {
  readonly id: string
  readonly label: string
  readonly source: string
  readonly stdin: string
}

export const samples: readonly PlaygroundSample[] = [
  {
    id: "hello-world",
    label: "Hello, Seseragi!",
    source: helloWorld,
    stdin: "",
  },
  {
    id: "rock-paper-scissors",
    label: "型付きじゃんけんCLI",
    source: rockPaperScissors,
    stdin: "rock\nscissors\n",
  },
]
