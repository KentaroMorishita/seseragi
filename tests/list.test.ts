import { describe, expect, test } from "bun:test"
import {
  applyList,
  arrayToList,
  bindList,
  Cons,
  concatList,
  Empty,
  headList,
  Just,
  type List,
  listToArray,
  mapList,
  Nothing,
  tailList,
} from "@seseragi/runtime"

// =============================================================================
// List型の基本操作テスト
// =============================================================================

describe("List型の基本操作", () => {
  describe("コンストラクタとパターンマッチング", () => {
    test("Empty リストの作成", () => {
      expect(Empty.tag).toBe("Empty")
    })

    test("Cons を使ったリスト作成", () => {
      const list = Cons(1, Cons(2, Cons(3, Empty)))
      expect(list.tag).toBe("Cons")
      if (list.tag === "Cons") {
        expect(list.head).toBe(1)
        expect(list.tail.tag).toBe("Cons")
        if (list.tail.tag === "Cons") {
          expect(list.tail.head).toBe(2)
        }
      }
    })

    test("単一要素のリスト", () => {
      const singleItem = Cons(42, Empty)
      expect(singleItem.tag).toBe("Cons")
      if (singleItem.tag === "Cons") {
        expect(singleItem.head).toBe(42)
        expect(singleItem.tail).toEqual(Empty)
      }
    })
  })

  describe("headList - リストの先頭要素", () => {
    test("空でないリストの先頭", () => {
      const list = Cons("a", Cons("b", Cons("c", Empty)))
      const head = headList(list)
      expect(head).toEqual(Just("a"))
    })

    test("空リストの先頭", () => {
      const head = headList(Empty)
      expect(head).toEqual(Nothing)
    })

    test("単一要素リストの先頭", () => {
      const list = Cons(999, Empty)
      const head = headList(list)
      expect(head).toEqual(Just(999))
    })
  })

  describe("tailList - リストの尻尾", () => {
    test("複数要素リストの尻尾", () => {
      const list = Cons(1, Cons(2, Cons(3, Empty)))
      const tail = tailList(list)
      expect(tail).toEqual(Cons(2, Cons(3, Empty)))
    })

    test("単一要素リストの尻尾", () => {
      const list = Cons(42, Empty)
      const tail = tailList(list)
      expect(tail).toEqual(Empty)
    })

    test("空リストの尻尾", () => {
      const tail = tailList(Empty)
      expect(tail).toEqual(Empty)
    })
  })
})

// =============================================================================
// List型の関数操作テスト
// =============================================================================

describe("List型の関数操作", () => {
  describe("mapList - リストの写像", () => {
    test("数値リストの変換", () => {
      const numbers = Cons(1, Cons(2, Cons(3, Empty)))
      const doubled = mapList(numbers, (x) => x * 2)
      expect(doubled).toEqual(Cons(2, Cons(4, Cons(6, Empty))))
    })

    test("文字列リストの変換", () => {
      const words = Cons("hello", Cons("world", Empty))
      const lengths = mapList(words, (s) => s.length)
      expect(lengths).toEqual(Cons(5, Cons(5, Empty)))
    })

    test("空リストの写像", () => {
      const empty: List<number> = Empty
      const result = mapList(empty, (x) => x * 2)
      expect(result).toEqual(Empty)
    })

    test("型変換の写像", () => {
      const numbers = Cons(42, Cons(100, Empty))
      const strings = mapList(numbers, (n) => n.toString())
      expect(strings).toEqual(Cons("42", Cons("100", Empty)))
    })
  })

  describe("concatList - リストの連結", () => {
    test("2つの非空リストの連結", () => {
      const list1 = Cons(1, Cons(2, Empty))
      const list2 = Cons(3, Cons(4, Empty))
      const result = concatList(list1, list2)
      expect(result).toEqual(Cons(1, Cons(2, Cons(3, Cons(4, Empty)))))
    })

    test("空リストとの連結", () => {
      const list = Cons("a", Cons("b", Empty))
      expect(concatList(Empty, list)).toEqual(list)
      expect(concatList(list, Empty)).toEqual(list)
    })

    test("両方とも空リストの連結", () => {
      const result = concatList(Empty, Empty)
      expect(result).toEqual(Empty)
    })

    test("複数リストの連鎖連結", () => {
      const list1 = Cons(1, Empty)
      const list2 = Cons(2, Empty)
      const list3 = Cons(3, Empty)
      const result = concatList(concatList(list1, list2), list3)
      expect(result).toEqual(Cons(1, Cons(2, Cons(3, Empty))))
    })
  })

  describe("applyList - アプリカティブ操作", () => {
    test("関数リストと値リストの適用", () => {
      const add1 = (x: number) => x + 1
      const mul2 = (x: number) => x * 2
      const functions = Cons(add1, Cons(mul2, Empty))
      const values = Cons(3, Cons(5, Empty))

      const result = applyList(functions, values)
      // [add1, mul2] <*> [3, 5] = [add1(3), add1(5), mul2(3), mul2(5)] = [4, 6, 6, 10]
      expect(result).toEqual(Cons(4, Cons(6, Cons(6, Cons(10, Empty)))))
    })

    test("空の関数リストとの適用", () => {
      const values = Cons(1, Cons(2, Empty))
      const result = applyList(Empty, values)
      expect(result).toEqual(Empty)
    })

    test("空の値リストとの適用", () => {
      const double = (x: number) => x * 2
      const functions = Cons(double, Empty)
      const result = applyList(functions, Empty)
      expect(result).toEqual(Empty)
    })

    test("単一関数と単一値", () => {
      const square = (x: number) => x * x
      const functions = Cons(square, Empty)
      const values = Cons(7, Empty)
      const result = applyList(functions, values)
      expect(result).toEqual(Cons(49, Empty))
    })

    test("カリー化関数での複数引数適用", () => {
      const add = (x: number) => (y: number) => x + y
      const partialApplied = applyList(Cons(add, Empty), Cons(10, Empty))
      const finalResult = applyList(partialApplied, Cons(5, Cons(7, Empty)))
      expect(finalResult).toEqual(Cons(15, Cons(17, Empty)))
    })
  })

  describe("bindList - モナディック操作", () => {
    test("基本的な bind 操作", () => {
      const list = Cons(1, Cons(2, Empty))
      const duplicate = (x: number): List<number> => Cons(x, Cons(x, Empty))

      const result = bindList(list, duplicate)
      expect(result).toEqual(Cons(1, Cons(1, Cons(2, Cons(2, Empty)))))
    })

    test("リストを展開する bind", () => {
      const list = Cons(3, Cons(4, Empty))
      const range = (n: number): List<number> => {
        if (n <= 0) return Empty
        return Cons(n, range(n - 1))
      }

      const result = bindList(list, range)
      // [3, 4] >>= range = range(3) ++ range(4) = [3, 2, 1] ++ [4, 3, 2, 1]
      expect(result).toEqual(
        Cons(3, Cons(2, Cons(1, Cons(4, Cons(3, Cons(2, Cons(1, Empty)))))))
      )
    })

    test("空リストとの bind", () => {
      const duplicate = (x: number): List<number> => Cons(x, Cons(x, Empty))
      const result = bindList(Empty, duplicate)
      expect(result).toEqual(Empty)
    })

    test("Empty を返す関数との bind", () => {
      const list = Cons(1, Cons(2, Empty))
      const alwaysEmpty = (_: number): List<number> => Empty
      const result = bindList(list, alwaysEmpty)
      expect(result).toEqual(Empty)
    })

    test("フィルタリング風の使用", () => {
      const numbers = Cons(1, Cons(2, Cons(3, Cons(4, Empty))))
      const evenOnly = (x: number): List<number> =>
        x % 2 === 0 ? Cons(x, Empty) : Empty

      const result = bindList(numbers, evenOnly)
      expect(result).toEqual(Cons(2, Cons(4, Empty)))
    })
  })
})

// =============================================================================
// 配列との相互変換テスト
// =============================================================================

describe("配列との相互変換", () => {
  describe("arrayToList - 配列からリストへ", () => {
    test("数値配列の変換", () => {
      const arr = [1, 2, 3, 4]
      const result = arrayToList(arr)
      expect(result).toEqual(Cons(1, Cons(2, Cons(3, Cons(4, Empty)))))
    })

    test("文字列配列の変換", () => {
      const arr = ["hello", "world"]
      const result = arrayToList(arr)
      expect(result).toEqual(Cons("hello", Cons("world", Empty)))
    })

    test("空配列の変換", () => {
      const arr: number[] = []
      const result = arrayToList(arr)
      expect(result).toEqual(Empty)
    })

    test("単一要素配列の変換", () => {
      const arr = [42]
      const result = arrayToList(arr)
      expect(result).toEqual(Cons(42, Empty))
    })
  })

  describe("listToArray - リストから配列へ", () => {
    test("数値リストの変換", () => {
      const list = Cons(5, Cons(10, Cons(15, Empty)))
      const result = listToArray(list)
      expect(result).toEqual([5, 10, 15])
    })

    test("文字列リストの変換", () => {
      const list = Cons("a", Cons("b", Cons("c", Empty)))
      const result = listToArray(list)
      expect(result).toEqual(["a", "b", "c"])
    })

    test("空リストの変換", () => {
      const result = listToArray(Empty)
      expect(result).toEqual([])
    })

    test("単一要素リストの変換", () => {
      const list = Cons("only", Empty)
      const result = listToArray(list)
      expect(result).toEqual(["only"])
    })
  })

  describe("相互変換の恒等性", () => {
    test("list -> array -> list", () => {
      const originalList = Cons(1, Cons(2, Cons(3, Empty)))
      const converted = arrayToList(listToArray(originalList))
      expect(converted).toEqual(originalList)
    })

    test("array -> list -> array", () => {
      const originalArray = [10, 20, 30]
      const converted = listToArray(arrayToList(originalArray))
      expect(converted).toEqual(originalArray)
    })

    test("空での相互変換", () => {
      expect(arrayToList(listToArray(Empty))).toEqual(Empty)
      expect(listToArray(arrayToList([]))).toEqual([])
    })
  })
})

// =============================================================================
// 実用的な使用例テスト
// =============================================================================

describe("実用的な使用例", () => {
  test("リスト内包表記風の操作", () => {
    // [(x, y) | x <- [1, 2], y <- [10, 20], x + y > 12]
    const xs = Cons(1, Cons(2, Empty))
    const ys = Cons(10, Cons(20, Empty))

    const result = bindList(xs, (x) =>
      bindList(ys, (y) => (x + y > 12 ? Cons([x, y], Empty) : Empty))
    )

    expect(result).toEqual(Cons([1, 20], Cons([2, 20], Empty)))
  })

  test("リストの平坦化", () => {
    const list1: List<number> = Cons(1, Cons(2, Empty))
    const list2: List<number> = Cons(3, Empty)
    const list3: List<number> = Cons(4, Cons(5, Empty))
    const nestedList: List<List<number>> = Cons(
      list1,
      Cons(list2, Cons(list3, Empty))
    )

    const flattened = bindList(nestedList, (innerList) => innerList)
    expect(flattened).toEqual(
      Cons(1, Cons(2, Cons(3, Cons(4, Cons(5, Empty)))))
    )
  })

  test("パイプライン処理", () => {
    const numbers = Cons(1, Cons(2, Cons(3, Cons(4, Empty))))

    // 偶数のみフィルタ -> 2倍 -> 文字列化
    const evenNumbers = bindList(numbers, (x) =>
      x % 2 === 0 ? Cons(x, Empty) : Empty
    )
    const doubled = mapList(evenNumbers, (x) => x * 2)
    const strings = mapList(doubled, (x) => `num: ${x}`)

    expect(strings).toEqual(Cons("num: 4", Cons("num: 8", Empty)))
  })

  test("リストの連鎖操作", () => {
    const words = Cons("hello", Cons("world", Empty))

    // 各単語を文字のリストに展開
    const chars = bindList(words, (word) => arrayToList(word.split("")))

    expect(listToArray(chars)).toEqual([
      "h",
      "e",
      "l",
      "l",
      "o",
      "w",
      "o",
      "r",
      "l",
      "d",
    ])
  })

  test("Maybe との組み合わせ", () => {
    const maybeNumbers = Cons(Just(1), Cons(Nothing, Cons(Just(3), Empty)))

    // Just から値を取り出してリストに展開
    const validNumbers = bindList(maybeNumbers, (maybeNum) =>
      maybeNum.tag === "Just" ? Cons(maybeNum.value, Empty) : Empty
    )

    expect(validNumbers).toEqual(Cons(1, Cons(3, Empty)))
  })
})
