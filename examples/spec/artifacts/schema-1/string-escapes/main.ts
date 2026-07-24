const classify = (value: string) => (($ssrg_match: string): string => $ssrg_match === "line\nnext" ? "matched" : "other")(value)
export const stringEscapeResults = (unit: undefined) => ["line\nnext", "tab\tvalue", "carriage\rreturn", "quote: \"Seseragi\"", "slash: \\", "lambda: λ", "nul:\0end", classify("line\nnext")]
