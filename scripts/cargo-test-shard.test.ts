import { expect, test } from "bun:test"
import {
  parseShard,
  parseTestList,
  shardTests,
  testExecutables,
} from "./cargo-test-shard"

test("shards are written as index/count", () => {
  expect(parseShard("2/4")).toEqual({ index: 2, count: 4 })
  for (const text of ["0/4", "5/4", "1/0", "1", "a/b", "1/2/3"]) {
    expect(() => parseShard(text)).toThrow()
  }
})

test("test lists keep only tests", () => {
  expect(
    parseTestList("a::first: test\r\nsecond: test\nbench: benchmark\n\n")
  ).toEqual(["a::first", "second"])
})

test("test executables are sorted by target and ignore other artifacts", () => {
  const line = (name: string, test: boolean, executable: string | null) =>
    JSON.stringify({
      reason: "compiler-artifact",
      profile: { test },
      target: { name, kind: ["test"] },
      executable,
    })
  const output = [
    line("zeta", true, "/t/zeta"),
    "not json",
    line("helper", false, "/t/helper"),
    line("alpha", true, "/t/alpha"),
    line("lib", true, null),
    JSON.stringify({ reason: "build-finished", success: true }),
  ].join("\n")
  expect(testExecutables(output)).toEqual([
    { name: "test:alpha", executable: "/t/alpha" },
    { name: "test:zeta", executable: "/t/zeta" },
  ])
})

test("every test runs in exactly one shard and shards stay balanced", () => {
  const targets = [
    { name: "big", tests: Array.from({ length: 258 }, (_, i) => `big_${i}`) },
    { name: "small", tests: ["s_b", "s_a", "s_c"] },
    { name: "shared", tests: ["big_1", "only_here"] },
  ]
  const count = 4
  const shards = Array.from({ length: count }, (_, i) =>
    shardTests(targets, { index: i + 1, count })
  )
  const all = shards.flat()
  expect(new Set(all).size).toBe(all.length)
  expect(new Set(all)).toEqual(
    new Set(targets.flatMap((target) => target.tests))
  )
  const sizes = shards.map((names) => names.length)
  expect(Math.max(...sizes) - Math.min(...sizes)).toBeLessThanOrEqual(1)
  for (const names of shards) {
    const big = names.filter((name) => name.startsWith("big_")).length
    expect(Math.abs(big - 258 / count)).toBeLessThanOrEqual(1)
  }
})

test("the split does not depend on the listing order", () => {
  const shard = { index: 2, count: 3 }
  expect(
    shardTests([{ name: "t", tests: ["c", "a", "b", "d"] }], shard)
  ).toEqual(shardTests([{ name: "t", tests: ["d", "b", "a", "c"] }], shard))
})
