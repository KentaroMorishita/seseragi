# Tourと記事surfaceの責務

この文書は、手を動かす教材を置くTourと、設計思想・内部・背景を読む記事surfaceの
境界を定めます。Tourの正本は[`curriculum.json`](./curriculum.json)、surface coverageは
[`coverage-report.md`](./coverage-report.md)です。

## 決定

Seseragiを学ぶためにsourceを読み、編集し、実行し、diagnosticを直す内容は、すべて通常の
Tour lessonとして扱います。lesson数や既存indexは互換性契約ではありません。必要な教材は
学習順として自然な位置または末尾へ追加し、stable lesson IDを保ったままdisplay order、
navigation、progress、prerequisite、generated artifactをdataから導出します。

`/deep-dive/`はURLを維持し、表示名を「Seseragi Articles」とします。ここは第二のcourseでは
なく、Type System、Abstraction Design、Runtime、Web、Architecture等のcategoryから記事を
探すsurfaceです。completion state、progress、履修prerequisite、番号付き経路、前後順を
持ちません。

## Tourに置く内容

次のいずれかを必要とする内容はTourへ置きます。

1. Seseragi sourceをeditorで読み、変更する。
2. RunからOutputまたはPreviewを確認する。
3. Exerciseを編集して別の結果を得る。
4. Compiler diagnosticから型・syntax・contractの誤りを直す。
5. 後続lessonの前提になるsyntax、type、operator、API、設計判断を学ぶ。

Generic / Trait / Functor / Applicative / Monadのlawや抽象境界も、codeを実行して学ぶ内容は
Tourです。これらを「理論だから」という理由で別progressへ移しません。

## Articlesに置く内容

Articlesは次の問いを扱います。

- なぜその公開contractを選んだか。
- Compilerやruntime内部で何が起きるか。
- Lawがどの置換・最適化・意味保存を支えるか。
- 別のarchitectureを選んだ場合のtrade-offは何か。
- 大規模化したとき、どこを観測し、どの境界を分けるか。

記事はstable URL、category、topic、関連Tour lesson、関連spec / Referenceへのlinkを持ちます。
Tour教材のsource、exercise、diagnostic、walkthrough本文を複製しません。実行が必要な箇所は
関連Tourへlinkし、記事本文は背景・内部・trade-offへ集中します。

## 教材追加の契約

- lesson IDとdisplay orderを分離する。
- Lesson数をliteralで固定しない。
- Category / chapter / lesson番号はcurriculum dataから導出する。
- 途中へlessonを追加しても、stable URLと保存済みcompleted IDはそのまま利用する。
- Navigationの前後関係、progress total、prerequisite検証は新しい配列順へ追従する。
- Generated manifestとcoverage reportはgeneratorで更新し、手書き件数を要求しない。
- 新lessonにはsource、output、exercise、diagnostic、walkthrough、introduced surface、recapを
  揃える。

## Surfaceごとの問い

| Surface | 問い | 正とする内容 |
| --- | --- | --- |
| Tour | 手を動かして何を学ぶか | source、Run、Output、exercise、diagnostic、学習順 |
| Articles | なぜそう設計したか | 設計思想、内部意味論、lawの背景、trade-off |
| Recipe | 既習概念を目的へどう適用するか | 一つの課題を解く実行可能な変形例 |
| Showcase | 統合すると何が作れるか | 完成applicationと探索可能なstate |
| Reference | 正確なsignatureや制約は何か | Compiler由来の型・API情報 |

同じtopicが複数surfaceへ現れても本文を二重管理しません。Tourは実行教材、Articlesは背景、
Recipe / Showcaseは利用例、Referenceはcompiler-owned identityを正とし、互いをlinkします。
