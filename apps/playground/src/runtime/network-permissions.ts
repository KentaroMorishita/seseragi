// 外部通信（外部の画像、HTTP、WebSocket、SSE）の許可（#1371）。
// - 許可は、例・自分のコードの作業場所・Tour のレッスンごとに、このブラウザに覚える。
// - 「許可しない」と答えたことも覚え、次からは聞かない。Settings から取り消せる。
// - 既定は止めたまま。利用者が一度はっきり許可したときだけ通す。
import type { EntryContract } from "../compiler/types"

export type NetworkDecision = "allowed" | "denied"

export type NetworkPermission = Readonly<{
  scope: string
  label: string
  decision: NetworkDecision
  decidedAt: string
}>

export type NetworkPermissionStore = Readonly<{
  get: (scope: string) => NetworkPermission | undefined
  set: (scope: string, label: string, decision: NetworkDecision) => void
  remove: (scope: string) => void
  list: () => readonly NetworkPermission[]
  subscribe: (listener: () => void) => () => void
}>

export const networkPermissionStorageKey = "seseragi.network-permissions.v1"

type PermissionStorage = Pick<Storage, "getItem" | "setItem">

export function createNetworkPermissionStore(
  storage: PermissionStorage,
  now: () => Date = () => new Date()
): NetworkPermissionStore {
  const listeners = new Set<() => void>()
  const read = (): NetworkPermission[] => {
    try {
      const parsed: unknown = JSON.parse(
        storage.getItem(networkPermissionStorageKey) ?? "[]"
      )
      return Array.isArray(parsed) ? parsed.filter(isPermission) : []
    } catch {
      return []
    }
  }
  const write = (permissions: readonly NetworkPermission[]): void => {
    try {
      storage.setItem(networkPermissionStorageKey, JSON.stringify(permissions))
    } catch {
      // 保存できない環境でも、この画面を開いているあいだは今の答えを使う。
    }
    memory = permissions
    for (const listener of listeners) listener()
  }
  let memory: readonly NetworkPermission[] = read()
  return {
    get: (scope) => memory.find((permission) => permission.scope === scope),
    set: (scope, label, decision) =>
      write([
        ...memory.filter((permission) => permission.scope !== scope),
        { scope, label, decision, decidedAt: now().toISOString() },
      ]),
    remove: (scope) =>
      write(memory.filter((permission) => permission.scope !== scope)),
    list: () => memory,
    subscribe: (listener) => {
      listeners.add(listener)
      return () => listeners.delete(listener)
    },
  }
}

function isPermission(value: unknown): value is NetworkPermission {
  if (typeof value !== "object" || value === null) return false
  const record = value as Record<string, unknown>
  return (
    typeof record.scope === "string" &&
    typeof record.label === "string" &&
    (record.decision === "allowed" || record.decision === "denied") &&
    typeof record.decidedAt === "string"
  )
}

// 実行の前に分かるもの: 実行の契約が、外部と通信するサービスを使う。
const networkServicePattern = /http|fetch|websocket|sse|event-?source/iu

export function contractNeedsNetwork(contract: EntryContract): boolean {
  return (contract.providers ?? []).some(
    ({ provider, service }) =>
      networkServicePattern.test(provider) ||
      networkServicePattern.test(service)
  )
}

// 静的な HTML の出力に、外部の画像や背景（http: / https:）がある。
const externalResourcePattern =
  /(?:\b(?:src|srcset|poster)\s*=\s*["']?\s*|url\(\s*["']?\s*)https?:\/\//iu

export function htmlNeedsNetwork(html: string): boolean {
  return externalResourcePattern.test(html)
}
