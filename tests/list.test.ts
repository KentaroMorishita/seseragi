import { describe, expect, test } from "bun:test"
import {
  type List,
  Empty,
  Cons,
  mapList,
  pureList,
  applyList,
  bindList,
  isEmpty,
  isCons,
  headList,
  tailList,
  lengthList,
  concatList,
  reverseList,
  takeList,
  dropList,
  filterList,
  foldLeftList,
  foldRightList,
  fromArray,
  toArray,
} from "../src/runtime/seseragi-runtime"

describe("List型", () => {
  describe("基本的なコンストラクタ", () => {
    test("Empty リストの作成", () => {
      const emptyList = Empty
      expect(emptyList.tag).toBe("Empty")
      expect(isEmpty(emptyList)).toBe(true)
    })

    test("Cons でリストの作成", () => {
      const list = Cons(1, Cons(2, Cons(3, Empty)))
      expect(list.tag).toBe("Cons")
      if (isCons(list)) {
        expect(list.head).toBe(1)
      }
      expect(isCons(list)).toBe(true)
      expect(isEmpty(list)).toBe(false)
    })
  })

  describe("基本操作", () => {
    test("headList - リストの先頭要素を取得", () => {
      const list = Cons(1, Cons(2, Cons(3, Empty)))
      expect(headList(list)).toBe(1)
      expect(headList(Empty)).toBeUndefined()
    })

    test("tailList - リストの末尾を取得", () => {
      const list = Cons(1, Cons(2, Cons(3, Empty)))
      const tail = tailList(list)
      expect(tail.tag).toBe("Cons")
      if (isCons(tail)) {
        expect(tail.head).toBe(2)
      }
      expect(tailList(Empty)).toEqual(Empty)
    })

    test("lengthList - リストの長さ", () => {
      expect(lengthList(Empty)).toBe(0)
      expect(lengthList(Cons(1, Empty))).toBe(1)
      expect(lengthList(Cons(1, Cons(2, Cons(3, Empty))))).toBe(3)
    })
  })

  describe("Functor法則 (mapList)", () => {
    test("fmap id = id", () => {
      const list = Cons(1, Cons(2, Cons(3, Empty)))
      const identity = <T>(x: T): T => x
      const mapped = mapList(list, identity)
      expect(toArray(mapped)).toEqual([1, 2, 3])
    })

    test("fmap (f . g) = fmap f . fmap g", () => {
      const list = Cons(1, Cons(2, Cons(3, Empty)))
      const f = (x: number): number => x * 2
      const g = (x: number): number => x + 1

      const composed = (x: number): number => f(g(x))
      const left = mapList(list, composed)
      const right = mapList(mapList(list, g), f)

      expect(toArray(left)).toEqual(toArray(right))
    })

    test("mapList で要素を変換", () => {
      const list = Cons(1, Cons(2, Cons(3, Empty)))
      const doubled = mapList(list, (x) => x * 2)
      expect(toArray(doubled)).toEqual([2, 4, 6])
    })
  })

  describe("Applicative法則", () => {
    test("pure関数でリストを作成", () => {
      const list = pureList(42)
      expect(toArray(list)).toEqual([42])
    })

    test("applicative apply で関数を適用", () => {
      const funcs = Cons(
        (x: number) => x * 2,
        Cons((x: number) => x + 1, Empty)
      )
      const values = Cons(1, Cons(2, Empty))
      const result = applyList(funcs, values)
      // [(*2), (+1)] <*> [1, 2] = [2, 4, 2, 3]
      expect(toArray(result)).toEqual([2, 4, 2, 3])
    })
  })

  describe("Monad法則 (bindList)", () => {
    test("left identity: return a >>= f = f a", () => {
      const a = 42
      const f = (x: number): List<number> => Cons(x, Cons(x * 2, Empty))

      const left = bindList(pureList(a), f)
      const right = f(a)

      expect(toArray(left)).toEqual(toArray(right))
    })

    test("right identity: m >>= return = m", () => {
      const list = Cons(1, Cons(2, Cons(3, Empty)))

      const left = bindList(list, pureList)
      const right = list

      expect(toArray(left)).toEqual(toArray(right))
    })

    test("associativity: (m >>= f) >>= g = m >>= (\\x -> f x >>= g)", () => {
      const list = Cons(1, Cons(2, Empty))
      const f = (x: number): List<number> => Cons(x, Cons(x + 10, Empty))
      const g = (x: number): List<number> => Cons(x * 2, Empty)

      const left = bindList(bindList(list, f), g)
      const right = bindList(list, (x) => bindList(f(x), g))

      expect(toArray(left)).toEqual(toArray(right))
    })

    test("bindList でリストを展開", () => {
      const list = Cons(1, Cons(2, Empty))
      const result = bindList(list, (x) => Cons(x, Cons(x * 2, Empty)))
      expect(toArray(result)).toEqual([1, 2, 2, 4])
    })
  })

  describe("リスト操作", () => {
    test("concatList - リストの連結", () => {
      const list1 = Cons(1, Cons(2, Empty))
      const list2 = Cons(3, Cons(4, Empty))
      const result = concatList(list1, list2)
      expect(toArray(result)).toEqual([1, 2, 3, 4])
    })

    test("reverseList - リストの反転", () => {
      const list = Cons(1, Cons(2, Cons(3, Empty)))
      const reversed = reverseList(list)
      expect(toArray(reversed)).toEqual([3, 2, 1])
    })

    test("takeList - 先頭からn個取得", () => {
      const list = Cons(1, Cons(2, Cons(3, Cons(4, Empty))))
      expect(toArray(takeList(2, list))).toEqual([1, 2])
      expect(toArray(takeList(0, list))).toEqual([])
      expect(toArray(takeList(5, list))).toEqual([1, 2, 3, 4])
    })

    test("dropList - 先頭からn個削除", () => {
      const list = Cons(1, Cons(2, Cons(3, Cons(4, Empty))))
      expect(toArray(dropList(2, list))).toEqual([3, 4])
      expect(toArray(dropList(0, list))).toEqual([1, 2, 3, 4])
      expect(toArray(dropList(5, list))).toEqual([])
    })

    test("filterList - 条件に合う要素をフィルタ", () => {
      const list = Cons(1, Cons(2, Cons(3, Cons(4, Empty))))
      const evens = filterList((x) => x % 2 === 0, list)
      expect(toArray(evens)).toEqual([2, 4])
    })
  })

  describe("fold操作", () => {
    test("foldLeftList - 左畳み込み", () => {
      const list = Cons(1, Cons(2, Cons(3, Empty)))
      const sum = foldLeftList((acc, x) => acc + x, 0, list)
      expect(sum).toBe(6)

      const concat = foldLeftList((acc, x) => acc + x.toString(), "", list)
      expect(concat).toBe("123")
    })

    test("foldRightList - 右畳み込み", () => {
      const list = Cons(1, Cons(2, Cons(3, Empty)))
      const sum = foldRightList((x, acc) => x + acc, 0, list)
      expect(sum).toBe(6)

      const concat = foldRightList((x, acc) => x.toString() + acc, "", list)
      expect(concat).toBe("123")
    })
  })

  describe("配列との相互変換", () => {
    test("fromArray - 配列からリストに変換", () => {
      const arr = [1, 2, 3, 4]
      const list = fromArray(arr)
      expect(toArray(list)).toEqual(arr)
    })

    test("toArray - リストから配列に変換", () => {
      const list = Cons(1, Cons(2, Cons(3, Empty)))
      const arr = toArray(list)
      expect(arr).toEqual([1, 2, 3])
    })

    test("空リストの変換", () => {
      expect(toArray(fromArray([]))).toEqual([])
      expect(fromArray(toArray(Empty))).toEqual(Empty)
    })
  })
})
