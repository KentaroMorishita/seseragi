# Native JSON file formatter

Read `input.txt`, parse exact JSON through a normal path dependency, and write
compact canonical JSON plus a newline to `output.txt`. Source order and arbitrary
precision numbers survive the conversion. Invalid syntax, duplicate object
fields and invalid UTF-8 leave the old output unchanged. The CLI acceptance test
checks original generated TypeScript in both profiles and runs Bun, Node and the
native binary without a runtime PATH.
