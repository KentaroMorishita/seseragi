# Native text and math data transformation

The path dependency trims comma-separated cells, skips empty/comment cells,
removes one literal `m`, parses Float values, and renders their square roots.
The root reads `input.txt` and writes `output.txt` only after every cell succeeds.
Invalid Float or UTF-8 input preserves the old output and original input.

This fixture exercises native Text split/trim/contains/replace and Math sqrt.
The source fixtures `basic-text-math` and `basic-text-math-effects` also cover
sin/cos/log, first-class functions, Unicode boundaries, cold replay and teardown.
The new operations remain explicitly refused by program WASM.
