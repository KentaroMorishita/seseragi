# Compiler artifact contracts

このdirectoryは、compiler stage間で受け渡すdebug / conformance artifactのschema fixtureです。
言語の意味やpublic ABIではなく、複数実装laneを疎結合に検証するためのversioned test contractです。

`schema-1/basic/` は同じsourceに対する次の四artifactを固定します。

- `tokens.json`: triviaとEOFを含むlossless token列
- `cst.json`: token rangeを参照するlossless CST骨格
- `diagnostics.json`: frontend共通diagnostic envelope
- `interface.json`: module consumerが読む公開interface

rangeはすべて0-based、end-exclusiveのUTF-8 byteです。tokenの`raw`をEOF以外すべて連結するとsourceと
byte単位で一致しなければなりません。CSTの`startToken` / `endToken`はtoken indexのhalf-open rangeです。

artifactのJSONは人が手で実装する入力ではありません。新compilerが生成し、conformance runnerが比較します。
schema fieldを変更するときはproducerとconsumerを同じ巨大changeへ混ぜず、schema fixture、producer、consumerの
順に移行します。schema majorが異なるartifactを黙って読み替えません。

`interface.json`はsource bodyやbackend layoutを含めず、公開symbol、type scheme、operator、instance、dependencyだけを
持ちます。private bodyを必要とする最適化は同一compiler runのTypedHir / CoreIrを使い、interface cacheへ漏らしません。
