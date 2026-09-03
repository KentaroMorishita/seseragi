/**
 * Internal, persistent radix index over nonnegative safe-integer addresses.
 * Patricia compression keeps exactly 2n - 1 nodes for n live leaves. A path
 * has at most 53 branches, independently of update history; point updates
 * copy only that path. Removed leaves leave neither tombstones nor parents.
 */
export type Index<Value> = Leaf<Value> | Branch<Value> | undefined

type Leaf<Value> = Readonly<{
  tag: "Leaf"
  min: number
  max: number
  size: 1
  value: Value
}>

type Branch<Value> = Readonly<{
  tag: "Branch"
  min: number
  max: number
  size: number
  bit: number
  left: NonNullable<Index<Value>>
  right: NonNullable<Index<Value>>
}>

const side = (key: number, bit: number): number =>
  Math.floor(key / 2 ** bit) % 2

function differingBit(left: number, right: number): number {
  const high = Math.floor(left / 2 ** 32) ^ Math.floor(right / 2 ** 32)
  return high === 0
    ? 31 - Math.clz32((left >>> 0) ^ (right >>> 0))
    : 63 - Math.clz32(high)
}

function branch<Value>(
  bit: number,
  left: NonNullable<Index<Value>>,
  right: NonNullable<Index<Value>>
): Branch<Value> {
  return Object.freeze({
    tag: "Branch",
    bit,
    left,
    right,
    min: left.min,
    max: right.max,
    size: left.size + right.size,
  })
}

export function indexGet<Value>(
  index: Index<Value>,
  key: number
): Value | undefined {
  let node = index
  while (node !== undefined) {
    if (key < node.min || key > node.max) return undefined
    if (node.tag === "Leaf") return node.value
    node = side(key, node.bit) === 0 ? node.left : node.right
  }
  return undefined
}

export function indexSet<Value>(
  index: Index<Value>,
  key: number,
  value: Value
): NonNullable<Index<Value>> {
  if (!Number.isSafeInteger(key) || key < 0) {
    throw new RangeError("persistent index address must be a nonnegative Int")
  }
  const leaf: Leaf<Value> = Object.freeze({
    tag: "Leaf",
    min: key,
    max: key,
    size: 1,
    value,
  })
  return put(index, leaf)
}

function put<Value>(
  index: Index<Value>,
  leaf: Leaf<Value>
): NonNullable<Index<Value>> {
  if (index === undefined) return leaf
  if (index.tag === "Leaf" && index.min === leaf.min) return leaf
  const bit = differingBit(index.min, leaf.min)
  if (index.tag === "Leaf" || bit > index.bit) {
    return side(leaf.min, bit) === 0
      ? branch(bit, leaf, index)
      : branch(bit, index, leaf)
  }
  return side(leaf.min, index.bit) === 0
    ? branch(index.bit, put(index.left, leaf), index.right)
    : branch(index.bit, index.left, put(index.right, leaf))
}

export function indexRemove<Value>(
  index: Index<Value>,
  key: number
): Index<Value> {
  if (index === undefined || key < index.min || key > index.max) return index
  if (index.tag === "Leaf") return undefined
  if (side(key, index.bit) === 0) {
    const left = indexRemove(index.left, key)
    if (left === index.left) return index
    return left === undefined
      ? index.right
      : branch(index.bit, left, index.right)
  }
  const right = indexRemove(index.right, key)
  if (right === index.right) return index
  return right === undefined ? index.left : branch(index.bit, index.left, right)
}

/** Find an unused address without a historical counter or a retained free list. */
export function indexVacant<Value>(index: Index<Value>): number {
  if (index === undefined || index.min > 0) return 0
  let node = index
  while (node.tag === "Branch" && node.size !== node.max - node.min + 1) {
    if (node.left.size !== node.left.max - node.left.min + 1) {
      node = node.left
    } else if (node.left.max + 1 < node.right.min) {
      return node.left.max + 1
    } else {
      node = node.right
    }
  }
  const vacant = node.max + 1
  if (!Number.isSafeInteger(vacant)) {
    throw new RangeError("persistent index address space is exhausted")
  }
  return vacant
}
