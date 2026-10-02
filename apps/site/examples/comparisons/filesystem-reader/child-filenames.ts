function addFilename(base: string, name: string): string {
  if (name === "" || name === "." || name === ".." || /[/\\\0]/.test(name)) {
    return `rejected: ${name}`
  }
  return `${base}/${name}`
}

for (const name of [
  "report.txt",
  "report.v2.txt",
  "../report.txt",
  "",
  ".",
  "..",
]) {
  console.log(addFilename("reports", name))
}

export {}
