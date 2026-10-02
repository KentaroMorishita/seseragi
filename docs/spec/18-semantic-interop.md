# 18. 外部型意味論の共通境界

## 18.1 目的と適用範囲

Semantic Interopは、外部の宣言やschemaから型の意味を読み取り、Seseragiの型、
ADT、Effect、resource規則に従うAPIへ変換するための共通境界です。
TypeScriptは最初の入力言語です。共通表現とSeseragi側の型検査は、
TypeScriptの構文木やcheckerの実装に依存してはなりません。

本章は各層の責務と変換時の判断を定義します。TypeScriptの具体的なraw bindingは
[7章](07-typescript-interop.md)、既定の変換方針と設定は
[8章](08-dts-conversion.md)に従います。本章の追加だけで、8.12の未対応型を
自動変換できるようになったとは扱いません。対応状況は実装とconformanceで別に確認します。

## 18.2 四つの層

| 層 | 入力と責務 | 出力 |
| --- | --- | --- |
| Source frontend | 入力言語の名前解決、公開symbol、型関係をその言語の規則で解釈する | 共通IRと元宣言への由来情報 |
| Interop Semantic IR | 入力言語に依存しない型関係、値の性質、実行時境界を保持し、参照と制約を検証する | Seseragiの構文から独立した意味のgraph |
| Seseragi projection | 共通IRと明示された変換方針から、既存のSeseragi型とadapterを構成する | binding候補、必要な変換、診断、変換report |
| Runtime linking | 解決済みの実行時identityと呼出規約に従い、対象hostの値へ接続する | 検査とlifetime管理を備えた実行時接続 |

Frontendはsource固有の構文木やchecker handleを内部で利用できます。ただし、
後続の層が意味を理解するために、その構文木を読み直す必要があってはなりません。
たとえば条件付き型は、解釈済みの型関係か未解決の意味状態として渡します。
TypeScriptのnode kindを共通IRの意味の定義にしません。

IRは外部の意味を表し、SeseragiのASTや生成文字列の別名にはしません。
同じIRから別の公開adapterを選べます。生成したSeseragi sourceは通常の
名前解決と型検査を通し、projection専用の型規則で検査を省略しません。

CLI、LSP、Playgroundは同じfrontendとprojectionを呼びます。
ファイルの読書き、package取得、仮想workspaceの管理は呼出側の責務です。
parse、型検査、format、Docs生成中に外部moduleのruntime codeを実行することは、
[6.10](06-modules-and-interop.md#610-初期化と評価)により禁止します。

## 18.3 型検査上の意味と実行時の意味

共通IRは、型検査上の関係と実行時の接続情報を別に保持します。
generic parameterのscopeとconstraint、型の投影、再帰参照、許容される値の集合は
型検査上の関係です。実行時のmodule identity、export path、receiver、呼出時の
arity、mutation、copy、throw、非同期処理、callbackの保持期間は接続情報です。

型の同値性だけでは、実行時の値を置換できると判断しません。型の形が一致しても、
別packageのopaque handleやmutable objectを同じidentityへ統合できません。
逆に、一つの実行時symbolを複数の公開名で参照する場合は、その同一性を保持します。
source位置やexport名の並び順を実行時identityとして使いません。

Frontendは宣言だけで確定できない性質を未確定として記録します。
戻り値が通常の値だからpureである、Promiseを返すからcancel可能である、
callback引数があるから同期呼出しだけである、と推測してはなりません。
TypeScriptでは7章と8章のtask既定、pure承認、callback設定を適用します。
別のfrontendも、不明な性質を無条件の保証へ変えてはなりません。

## 18.4 意味の保持、opaque、明示方針

Projectionは公開symbolごとに変換結果と理由を記録します。
自動変換では、許容される入力、結果、失敗、mutation、identity、lifetimeのうち、
公開APIに関係する条件を保持しなければなりません。境界検査やcopyが必要なら、
その処理をadapterとして生成し、zero-costの型aliasとは区別します。

外部の値を分解せずに受け渡せる場合は、opaque型として保持できます。
その型から公開する操作にも有効な呼出規約が必要です。未解釈の型をopaqueという
名前に変えるだけでは、受渡しや操作が安全だと判断できません。
解釈できた意味、隠した構造、利用可能な操作をreportに分けて記録します。

意味を保持する変換が一意に決まらなければ、binding authorの明示方針を要求します。
方針は対象symbol、適用rule、入力条件、生成する検査、残る制限を特定し、
設定digestとともに記録します。TypeScriptのnumberからIntへの変換では、
finite・integral・safe integerの実行時検査が必要です。設定だけで検査を省きません。

解釈できない意味、未実装の変換、未選択の方針、実行先で利用できない値は、
理由を区別してreportへ出します。必要な条件が未確定のsymbolは生成を拒否します。
`Any`や`Unknown`へ黙って弱めたり、unionの一部やgeneric constraintを捨てたり
して成功としてはなりません。8章の明示fallbackを選んだ場合も、元の意味を
失った箇所と必要なdecoderを記録し、意味を保持した自動変換とは区別します。

## 18.5 Seseragiへの構成とruntime接続

Projectionは、外部の構造をSeseragiの既存のstructural record、nominal型、
opaque型、ADT、Effectへ明示的に対応付けます。外部の構文を一対一で写すことや、
似た型名だけでtraitを選ぶことを変換規則にしません。
unionからADTを生成する場合は、識別値を検査するdecoderと元の表現を復元する
encoderが必要です。未知のvariantは、宣言した失敗型で呼出側へ返します。

Runtime linkingは元ecosystemの実装を利用します。型変換のために外部の
実装をSeseragiで再実装する必要はありません。host resolverが返すexact identity、
対象platform、moduleの評価規則、calling conventionを接続時に検証します。
browser、Node、Bunで同じspecifierが書けることだけでは利用可能と判断しません。

resourceを所有するhandleや保持されるcallbackは、取得・解放・cancellationの
明示契約に従います。型が変換できても、解放方法が不明な操作を通常の関数として
公開しません。Effect、defect、cancellationの区別は5章と7章に従います。

## 18.6 由来情報と拡張契約

Frontendは元sourceのidentity、symbol、source range、解釈ruleを記録します。
Projectionは利用したIR要素、方針、生成symbolへの対応を追記します。
複数宣言から一つの意味を構成するときも、関係する由来を保持します。
診断は失敗した層と対象を示し、CLIとPlaygroundで同じ根拠を表示できるものとします。

IRとreportはversion付きで決定的に保存できなければなりません。
未対応versionや意味の分からない必須要素を読み飛ばして生成を続けません。
再生成では入力、方針、frontend、projection、runtime契約のidentityを記録し、
互換性の変化を検出します。具体的なschemaと互換性判定はこの責務を保って定義します。

別frontendを追加する場合は、source固有の名前解決をそのfrontendに閉じます。
共通IRの既存要素で表せる意味を使い、追加要素が必要なら意味と検証規則を先に定義します。
source固有の補足情報を保持しても、projectionがそれを解釈しないと正しく動かない
構造にはしません。別sourceによるfixtureで、同じIR検証とprojectionが利用できることを
確認します。sourceの意味、変換後の型、実際のruntime挙動はそれぞれ検証します。
