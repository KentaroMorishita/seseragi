const original = "e\u0301"
const composed = original.normalize("NFC")
console.log(`NFC: ${composed}
NFD scalars: ${Array.from("é".normalize("NFD")).length}
NFKC: ${"ﬃ①".normalize("NFKC")}
NFKD: ${"①".normalize("NFKD")}
original scalars: ${Array.from(original).length}
lowercase: ${"Straße".toLowerCase()}
uppercase: ${"Straße".toUpperCase()}`)
