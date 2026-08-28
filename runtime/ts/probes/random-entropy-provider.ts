import { length as bytesLength } from "@seseragi/runtime/bytes"
import { entropySize, secureBytes } from "@seseragi/runtime/entropy"
import { run } from "@seseragi/runtime/effect"
import { createProviderEntropy } from "@seseragi/runtime/provider-entropy"
import { createProviderRandom } from "@seseragi/runtime/provider-random"
import { ProviderPackageLoader } from "@seseragi/runtime/provider-package"
import {
  algorithmId,
  intBetween,
  nextInt,
  randomBytes,
  randomSize,
  shuffle,
} from "@seseragi/runtime/random"
import { createEntropyProvider as createRawEntropyProvider } from "./node_modules/seseragi/entropy.ts"

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message)
}

process.env.SESERAGI_RANDOM_SEED = "42"
const loader = new ProviderPackageLoader("bun-process", [
  {
    provider: "seseragi/runtime#random",
    service: "std/random::Random",
    target: "bun-process",
    module: "seseragi/runtime-bun/random",
    exportName: "provider",
    loadMode: "eager",
    importModule: () => import("seseragi/runtime-bun/random"),
  },
  {
    provider: "seseragi/runtime-bun#entropy",
    service: "std/entropy::Entropy",
    target: "bun-process",
    module: "seseragi/runtime-bun/entropy",
    exportName: "provider",
    loadMode: "eager",
    importModule: () => import("seseragi/runtime-bun/entropy"),
  },
])

await loader.start()
const random = createProviderRandom(await loader.load("seseragi/runtime#random"))
const entropy = createProviderEntropy(
  await loader.load("seseragi/runtime-bun#entropy")
)
const randomEnvironment = { random }
const entropyEnvironment = { entropy }

const id = await run(algorithmId(), randomEnvironment)
assert(id.kind === "success" && id.value === "seseragi-xoshiro256ss-v1", "algorithm ID drifted")
const first = await run(nextInt(), randomEnvironment)
const second = await run(nextInt(), randomEnvironment)
assert(first.kind === "success" && first.value === 755370490430936, "first seed output drifted")
assert(second.kind === "success" && second.value === 3413550631330343, "second seed output drifted")

for (let index = 0; index < 256; index += 1) {
  const sampled = await run(intBetween(-7, 13), randomEnvironment)
  assert(sampled.kind === "success", "valid range failed")
  assert(sampled.value >= -7 && sampled.value < 13, "range sample escaped bounds")
}
const permutation = await run(shuffle([0, 1, 2, 3, 4]), randomEnvironment)
assert(permutation.kind === "success", "shuffle failed")
assert(
  [...permutation.value].sort((left, right) => left - right).join(",") === "0,1,2,3,4",
  "shuffle did not preserve values"
)
const pseudoSize = randomSize(17)
assert(pseudoSize.tag === "Right", "valid random size failed")
const pseudo = await run(randomBytes(pseudoSize.value), randomEnvironment)
assert(pseudo.kind === "success" && bytesLength(pseudo.value) === 17, "random bytes length drifted")

const secureSize = entropySize(32)
assert(secureSize.tag === "Right", "valid entropy size failed")
const secure = await run(secureBytes(secureSize.value), entropyEnvironment)
assert(secure.kind === "success" && bytesLength(secure.value) === 32, "secure bytes failed")

const failingEntropy = createProviderEntropy({
  provider: "fixture/runtime#entropy",
  service: "std/entropy::Entropy",
  entry: createRawEntropyProvider("fixture/runtime#entropy", ["bun-process"], {
    fill: () => {
      throw new Error("fixture entropy read failure")
    },
  }),
})
const failedSecure = await run(secureBytes(secureSize.value), {
  entropy: failingEntropy,
})
assert(
  failedSecure.kind === "failure" &&
    failedSecure.error.tag === "EntropyReadFailure",
  "host CSPRNG failure must stay in the typed failure channel"
)

const unavailableEntropy = createProviderEntropy({
  provider: "fixture/runtime#entropy-unavailable",
  service: "std/entropy::Entropy",
  entry: createRawEntropyProvider(
    "fixture/runtime#entropy-unavailable",
    ["bun-process"],
    {
      available: () => false,
      fill: () => {
        throw new Error("unavailable entropy must not attempt a read")
      },
    }
  ),
})
const unavailableSecure = await run(secureBytes(secureSize.value), {
  entropy: unavailableEntropy,
})
assert(
  unavailableSecure.kind === "failure" &&
    unavailableSecure.error.tag === "EntropyUnavailable",
  "missing host CSPRNG must stay in the typed failure channel"
)

await loader.shutdown()
process.stdout.write("random and entropy provider probe passed\n")
