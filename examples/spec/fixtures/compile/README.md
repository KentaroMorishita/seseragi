# Compile fixtures

実行を必要としないpositive parse / type-check caseを置きます。一fileにつき中心機能を一つに絞ります。
各sourceは同名の `.expect.json` で `kind: "compile"` と根拠spec sectionを宣言します。
