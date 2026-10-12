> 2026-10-10: 旧記事・URL・ページ数の保全条件は撤回済み。現在の制作方針は
> [編集契約](editorial-contract.md)、公開構成と検証は[制作記録](migration-ledger.md)、
> 作業状態は[#601](https://github.com/KentaroMorishita/seseragi/issues/601)。
> このフォルダの以前のwork-item/verification資料とinventoryは実装史であり、
> 旧記事の移植や保全を義務付ける現行gateではない。

# Docs Reboot design and evidence

[#601](https://github.com/KentaroMorishita/seseragi/issues/601) is the single
entry point for dependencies and acceptance. Begin with #763; the old leaf
issues and orchestration order are implementation history, not a work queue.

## Authoritative documents

- [Editorial contract](editorial-contract.md): the sole policy for audience,
  conversational prose, code aesthetics, article/chapter composition, API depth
  and visual direction. Start every new Docs batch here.
- [Migration ledger](migration-ledger.md) and [route snapshot](migration-inventory.tsv):
  KEEP/REWRITE/RETIRE/MERGE decisions, source/route owners, replacement and deletion
  prerequisites, compatibility and validation evidence.
- [function-chapter-review.md](function-chapter-review.md): #765の章候補、作者・実読者の確認対象と独立した受け入れ状態。
- [Reader review protocol](reader-contract.md): how to distinguish technical
  checks, author decisions, agent self-review and actual reader understanding.
- [Site architecture](site-architecture.md): global surfaces and navigation,
  with the new [entrance/pilot map](site-architecture.md#12-entrance-and-pilot-page-map).
- [Implementation architecture](implementation-architecture.md): typed Seseragi
  pages/locales, pure Html/process SSG, Compiler Reference, external-source and
  build/deployment boundaries.
- [Reference content map](reference-content-map.md): semantic provenance for
  existing and planned routes; not a published-corpus or prose-acceptance claim.

Language meaning remains in `docs/spec/`; symbol identity and signatures remain
compiler-owned. Executable canonical/site sources ground all behavior claims.
English keeps unprefixed routes and Japanese mirrors the same identity under
`/ja/`. Site content, navigation, layout and rendering remain Seseragi-owned;
TypeScript bridges external build/browser data. Playground remains independent.

## Previous results and mockups

`reader-review.md`, `*-verification.md`, `*-work-item.md` and previous reading
sessions retain their recorded source, execution, feedback and implementation
history. They are not new dispatch instructions or blanket Docs Reboot approval.
Do not erase unfinished work or reinterpret an old checkbox as new acceptance.

`seseragi-home.png`, `docs-language-function-application.png` and
`docs-standard-library-get.png` are historical design references. Their hierarchy
and reusable UI ideas may inform #764/#766; they do not override the editorial
contract or certify the new Hero/content composition.

The first quality model is #765 Functions/Notation, after #763 and the necessary
#764 composition. #766 uses that example; #767 bulk migration follows the accepted
chapter. #768 adds optional interaction and does not block the chapter/home.
#702/#706/#740/#631 remain independent gates under #601.
