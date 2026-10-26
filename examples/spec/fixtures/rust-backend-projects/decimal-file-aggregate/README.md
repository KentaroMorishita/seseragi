# 数値ファイルの合計

input.txtの一行一つのdecimal Intをparseし、成功時だけ合計とLFをoutput.txtへ書く。入力は変更しない。空入力は0、`20\n22\n`は`42\n`となる。canonical decimal以外や範囲外は型付きエラー、合計overflowはDefectとなり、どちらも旧output.txtを保持する。

通常path dependencyのnumbersへ集計とエラー表示を分け、rootのFileSystem Effectから呼ぶ。stdのIntParseError Showは既知の未対応#964として残し、ここではconstructorを明示matchする。parseRadix、全numeric API、一般provider ABIの対応を意味しない。

```sh
seseragi experimental build-native . --entry main --out-dir native
```
