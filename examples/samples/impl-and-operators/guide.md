inherent methodと`operator +`をStructの`impl`へまとめます。演算子は個別runtime分岐ではなく、標準Trait instanceへloweringされます。
