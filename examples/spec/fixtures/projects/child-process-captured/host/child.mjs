const chunks = [];
for await (const chunk of process.stdin) {
  chunks.push(chunk);
}

const input = Buffer.concat(chunks).toString("utf8");
process.stdout.write(input.toUpperCase());
process.stderr.write("warn");
process.exitCode = 7;
