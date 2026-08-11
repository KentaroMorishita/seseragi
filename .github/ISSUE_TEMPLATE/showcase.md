---
name: Showcase
about: Add a reviewed canonical Web Showcase to the Playground
title: "Showcase: "
labels: ""
assignees: ""
---

## Purpose

このShowcaseで初見の利用者へ伝える中心機能・世界観を記述する。

## Quality contract

- [ ] [`docs/SHOWCASE_QUALITY.md`](../../docs/SHOWCASE_QUALITY.md)を読んだ
- [ ] Issue #245のfull-page screenshotとsource zipの両方を確認した
- [ ] first view、typography、layout rhythm、visual identityを説明できる
- [ ] initial以外の主要interaction stateを設計した
- [ ] visual sectionとcomponent / state / named styleの境界が対応している
- [ ] generic card-grid、placeholder、既存Showcaseのvisual identityの複製ではない

## Review evidence

- [ ] `showcase-review.json`へ短いdesign intentを記録した
- [ ] desktop first viewとfull flow / 主要stateをhuman reviewした
- [ ] iPhone 390pxとAndroid 360pxのflowをhuman reviewした
- [ ] interaction後の代表stateをhuman reviewした
- [ ] Previewとmobile Code surfaceをhuman reviewした
- [ ] human approval後の状態だけを#219 visual baselineへ記録した

CI greenだけではこのIssueをcloseしない。
