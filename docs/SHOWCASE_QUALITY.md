# Showcase quality contract

PlaygroundのShowcaseは、機能がcompile・interactionするだけのfixtureではなく、
初見の利用者へ「Seseragiでここまで作れる」と示す完成例です。新規Web Showcaseは、
次の二つを実際に確認してから設計・reviewします。

- [full-page screenshot](https://github.com/user-attachments/assets/2a77e71a-9060-43a6-ae19-f1293ff938e2)
- [source zip](https://github.com/user-attachments/files/30919541/seseragi-landing-page.zip)

referenceの色やlayoutをコピーする必要はありません。first viewからclosingまでのrhythm、
typography・image・code panelの優先順位、desktopとmobileそれぞれのcomposition、そして
visual sectionとsource moduleの責務が対応している完成度を基準にします。

## Quality checklist

### First view

- 最初のviewportだけで中心機能と世界観が分かる。
- heading、supporting copy、primary visual / controlの優先順位が明確である。
- 空白を埋めるだけのcard、badge、tagやgeneric dashboard compositionを使わない。

### Typography

- display、section heading、body、metadataの役割が見た目から区別できる。
- line-heightとline lengthを含め、desktop / mobileそれぞれで読みやすい。
- mobileでcatch copyを不自然な一文字・一単語だけの行へ落とさない。

### Layout / spacing

- component内とsection間のspacingに階層があり、page全体に疎密のrhythmがある。
- すべてを同じborder / radiusのcardへ押し込まない。
- desktop gridを縮小するだけでなく、mobileのsection flowを設計する。

### Visual identity

- 題材に合う色、形、image、motionの言語がpage全体で一貫する。
- 既存Showcaseのpurple hero / white cardを無条件に複製しない。
- imageを使う場合は背景埋めではなく、section composition上の役割を持たせる。

### Interaction state

- initialだけでなく主要なhover / focus / active / selected / disabled / empty /
  error stateでもhierarchyを保つ。
- state変化でlayout jumpやhorizontal overflowを起こさない。
- keyboardとtouchで同じ中心機能へ到達できる。

### Code readability

- visual sectionとcomponent / state / composition / named styleの境界が対応する。
- entry pointはquery・page Signal・`dom.run`の接続へ責務を絞る。
- 長大な一行style、意味不明なtop-level value、compiler workaroundで見た目を作らない。
- mobile Code surfaceでもpage構造と主要stateの所有者を追える。

添付sourceのfile構成は強制しません。single-fileでも題材に十分なら使えますが、巨大な
一枚sourceへ押し込む理由にはしません。`main.ssrg`、page composition、visual component、
state / Action / dispatch、named style、`String -> WebUrl`のような変換責務を、題材に応じた
読める境界へ分けます。

## Required human review

CI greenだけでは承認しません。各Showcaseの`showcase-review.json`へdesign intentを記録し、
次を人間が実物で確認してから`approval.status`を`approved`にします。

- desktop first viewとfull flowまたは主要state
- iPhone 390px相当のfirst view
- Android 360px相当のnarrow mobile flow
- initial以外の代表interaction state
- Previewとmobile Code surface
- generic templateではなく、題材固有のvisual decisionがあること
- source structureがvisual qualityのために犠牲になっていないこと

review artifactはFirst view、Layout rhythm、Visual identity、Interaction、Code structureの
意図を短い文章で説明し、確認したviewport、state、surface、evidenceを列挙します。
自動testはartifactの存在と必須項目を検証しますが、文章の妥当性を代行しません。

## Baseline workflow

1. 実ブラウザで上記human reviewを行い、evidenceを保存する。
2. `showcase-review.json`を`approved`にし、focused contract testを通す。
3. 承認後の画面だけを`bun run test:visual:update -- "<review reason>"`で更新する。
4. `e2e/visual-baselines.review.json`へPNGとShowcase review artifact双方のSHA-256を固定する。

baselineは「良さ」を判定する仕組みではなく、承認済み状態からの意図しない差分を検出する
仕組みです。generic card-gridやplaceholderをbaselineへ入れてからCI greenを完了根拠にする
ことはできません。

## Application to #244

#244のlanding page Showcaseは、この契約を適用する最初のreference implementationです。
実装・review担当者はIssue #245の文章だけで済ませず、上記screenshotとsource zipの両方を
確認します。strong hero、page全体のsection rhythm、mobile固有のflow、code presentation、
official identity、visual/source structureの対応を`showcase-review.json`へ記録し、human
approval後にだけ#219 baselineを更新します。

既存Showcaseは#188、#189、#190、#191のreview evidenceを本契約へ紐付けています。
それらのvisual styleを新規Showcaseのtemplateとして扱わず、題材ごとのidentityを設計します。
