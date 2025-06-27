# Array型とList型シンタックスシュガー実装プラン

## 目標設計
```seseragi
// Array型（TypeScript互換、可変、インデックスアクセス）
let array = [1, 2, 3]
let element = array[0]

// List型（関数型、不変、構造的操作）
let empty = `[]                    // Empty
let list = `[1, 2, 3]             // Cons 1 (Cons 2 (Cons 3 Empty))
let consed = 1 : 2 : 3 : `[]      // Cons構文
let mixed = 1 : `[2, 3]           // 既存リストに追加
```

## Phase 1: Array型の実装

### 1.1 AST拡張
- `ArrayLiteral` ノード追加
- `ArrayAccess` ノード追加（`array[index]`構文）

### 1.2 パーサー拡張
- `[1, 2, 3]` 構文の解析
- `array[index]` アクセス構文の解析
- List構文との明確な区別

### 1.3 型推論拡張
- `Array<T>` 型の追加
- 配列リテラルの型推論
- インデックスアクセスの型検証

### 1.4 コード生成拡張
- `[1, 2, 3]` → `[1, 2, 3]` (TypeScript配列そのまま)
- `array[index]` → `array[index]` (ネイティブアクセス)

## Phase 2: List型シンタックスシュガー

### 2.1 Lexer拡張
- `BACKTICK` トークン追加 (`` ` ``)
- `COLON` トークンの演算子としての認識
- `CONS` 演算子の右結合設定

### 2.2 Parser拡張
- `` `[...] `` 構文の解析
- `: ` 演算子の実装（右結合）
- 脱糖処理：新構文 → 既存Cons/Empty

### 2.3 構文変換ルール
```
`[]                → Empty
`[1, 2, 3]        → Cons 1 (Cons 2 (Cons 3 Empty))
1 : 2 : `[]       → Cons 1 (Cons 2 Empty)
1 : `[2, 3]       → Cons 1 (Cons 2 (Cons 3 Empty))
```

### 2.4 既存システム活用
- 型推論：既存のCons/Empty型推論をそのまま使用
- コード生成：既存のList実装をそのまま使用
- ランタイム：変更なし

## Phase 3: Array↔List相互運用

### 3.1 変換関数の追加
```seseragi
fn arrayToList arr :Array<a> -> List<a>
fn listToArray lst :List<a> -> Array<a>
```

### 3.2 ランタイム実装
- 効率的な相互変換実装
- TypeScript側での最適化

### 3.3 使用例
```seseragi
let scores = [85, 92, 78]           // Array<Int>
let scoreList = arrayToList scores  // List<Int>
let processed = map (+10) scoreList // List関数型操作
let result = listToArray processed  // Array<Int>
```

## Phase 4: 高度な機能

### 4.1 リスト内包表記（将来）
```seseragi
`[x * 2 | x <- `[1, 2, 3], x > 1]  // List comprehension
```

### 4.2 パターンマッチング拡張
```seseragi
match someList {
  `[] -> "empty"
  x : `[] -> "singleton"  
  x : y : xs -> "multiple"
}
```

## 実装優先順位

1. **Phase 1**: Array型基本実装（TypeScript互換性重視）
2. **Phase 2**: List型シンタックスシュガー（美しい構文）
3. **Phase 3**: 相互運用（実用性向上）
4. **Phase 4**: 高度な機能（段階的拡張）

## 技術的考慮点

- 既存のList実装との完全互換性
- TypeScript生成コードの効率性
- パーサーの曖昧性回避
- エラーメッセージの明確化
- パフォーマンス最適化