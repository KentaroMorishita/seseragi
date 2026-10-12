# Docs Reboot レビューの区別

編集判断は [editorial-contract.md](editorial-contract.md)、現在の制作・技術検証は
[migration-ledger.md](migration-ledger.md)、作業入口は[#601](https://github.com/KentaroMorishita/seseragi/issues/601)。
旧本文・URL・ページ数・anchorの保全は不要。旧corpusの検証記録は過去の実装史。

## 記事を通読する

- 小さな仕事とコードから始まり、結果を確かめて、新しい記法の意味へ進めるか。
- 関数は小さく自然な名前か。記法を見せるためだけのpipelineや全部盛りがないか。
- 記法の読み方と選んだ理由が分かるか。コードの逐語翻訳に陥っていないか。
- 日英が同じsource・条件・結果を扱い、各言語で自然に話しているか。
- 詳しい規則を直接引けるか。章の入口と次に試す出口があるか。

## 証拠を混ぜない

| 確認 | 記録 |
| --- | --- |
| Compiler/実行 | source、型、stdout/診断、toolchain、build結果 |
| Browser | 全公開route/API宣言、日英切替、mobile/desktop、JS有効/無効、リンク、screenshot |
| Agent編集レビュー | 通読して気付いた点と修正内容 |
| 作者の美学 | 作者が実際に示した判断と対象範囲 |
| 初見読者の理解 | 読者の背景、分かったこと、必要だった助け |

Agentの検証を作者や実読者の承認へ読み替えない。
未確認の軸は未確認と書く。ただし通常の実装・PR・マージ・検証済みDocs公開は
都度の承認待ちで止めず継続する。全APIに長い読み物や個別承認を要求しない。
