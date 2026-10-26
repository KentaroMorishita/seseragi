# Integer remainder file transform

Run the source or this ordinary path package with an `input.txt` in the working
directory. Each decimal integer is replaced by its remainder on division by 3;
the complete result is written to `output.txt` after reading and parsing succeeds.
Input `7\n-7\n8\n` produces `1\n-1\n2\n`. Empty input produces an empty file.
Read and parse failures preserve an existing output file.

The native and original strict TypeScript programs use the same locked source.
Int remainder follows the sign of its left operand and normalizes zero. Division
by zero is a language Defect. This fixture does not expand program WASM arithmetic
or introduce new checked arithmetic APIs.
