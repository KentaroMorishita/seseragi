`struct Profile`は名前を持つnominalな型です。`Profile { ... }`で値を組み立て、`.`に続けてfield名を書くと中の値を読めます。

`{ prefix: String, suffix: String }`は名前を持たないstructural Record型です。StructとRecordのspreadは、元の値を変更せず、指定したfieldを置き換えた新しい値を返します。
